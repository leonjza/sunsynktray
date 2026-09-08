#!/usr/bin/env bash

set -euo pipefail

pem_file=""
cleanup() {
  if [[ -n "$pem_file" ]]; then
    rm -f -- "$pem_file"
  fi
}
trap cleanup EXIT

# These fallback values are a disposable demo account used only for the API
# smoke test. Production credentials must be supplied through the environment.
USERNAME="${SUNSYNK_USERNAME:-virtual@e-linter.com}"
PASSWORD="${SUNSYNK_PASSWORD:-elinter@0512}"
BASE_URL="${SUNSYNK_BASE_URL:-https://api.sunsynk.net}"
SOURCE="sunsynk"
CLIENT_ID="csp-web"
GRANT_TYPE="password"

for bin in curl jq openssl base64; do
  command -v "$bin" >/dev/null 2>&1 || { echo "Missing required binary: $bin"; exit 1; }
done

md5hex() {
  if command -v md5sum >/dev/null 2>&1; then
    printf "%s" "$1" | md5sum | awk '{print $1}'
  elif command -v md5 >/dev/null 2>&1; then
    printf "%s" "$1" | md5 | awk '{print $NF}'
  else
    echo "No md5 or md5sum found" >&2
    exit 1
  fi
}

b64_oneline() {
  base64 | tr -d '\n'
}

encrypt_password() {
  # Prefer pkeyutl (OpenSSL 3+) to avoid rsautl deprecation; fall back if missing.
  if openssl pkeyutl -help >/dev/null 2>&1; then
    printf "%s" "$PASSWORD" \
      | openssl pkeyutl -encrypt -pubin -inkey "$pem_file" -pkeyopt rsa_padding_mode:pkcs1 \
      | b64_oneline
  else
    printf "%s" "$PASSWORD" \
      | openssl rsautl -encrypt -pubin -inkey "$pem_file" \
      | b64_oneline
  fi
}

now_ms() {
  echo "$(( $(date +%s) * 1000 ))"
}

utc_date_days_ago() {
  local days_ago="$1"
  if date -u -v-"${days_ago}"d +%F >/dev/null 2>&1; then
    date -u -v-"${days_ago}"d +%F
  else
    date -u -d "${days_ago} days ago" +%F
  fi
}

echo "[*] Using BASE_URL=${BASE_URL}"
echo "[*] Fetching public key..."

nonce_pub=$(now_ms)
sign_pub_input="nonce=${nonce_pub}&source=${SOURCE}POWER_VIEW"
sign_pub=$(md5hex "$sign_pub_input")

pubkey_json=$(curl -sS "${BASE_URL}/anonymous/publicKey?nonce=${nonce_pub}&source=${SOURCE}&sign=${sign_pub}")
pubkey_b64=$(echo "$pubkey_json" | jq -er '.data')

echo "[+] Public key length: ${#pubkey_b64}"

echo "[*] Encrypting password..."
pem_file=$(mktemp)
{
  echo "-----BEGIN PUBLIC KEY-----"
  echo "$pubkey_b64" | fold -w 64
  echo "-----END PUBLIC KEY-----"
} > "$pem_file"

encrypted_pwd_b64=$(encrypt_password)

login_nonce=$(now_ms)
pubkey_prefix=${pubkey_b64:0:10}
sign_login_input="nonce=${login_nonce}&source=${SOURCE}${pubkey_prefix}"
sign_login=$(md5hex "$sign_login_input")

echo "[*] Logging in..."
login_payload=$(jq -cn \
  --arg sign "$sign_login" \
  --argjson nonce "$login_nonce" \
  --arg username "$USERNAME" \
  --arg password "$encrypted_pwd_b64" \
  --arg grant_type "$GRANT_TYPE" \
  --arg client_id "$CLIENT_ID" \
  --arg source "$SOURCE" \
  '{sign: $sign, nonce: $nonce, username: $username, password: $password, grant_type: $grant_type, client_id: $client_id, source: $source}')
login_response=$(curl -sS -X POST "${BASE_URL}/oauth/token/new" \
  -H "Content-Type: application/json;charset=UTF-8" \
  -d "$login_payload")

access_token=$(echo "$login_response" | jq -er '.data.access_token')
echo "[+] Login successful; token length: ${#access_token}"

auth_header=("Authorization: Bearer ${access_token}")

date_utc=$(date -u +%F)

echo "[*] Fetching plants..."
plants_json=$(curl -sS -H "${auth_header[@]}" "${BASE_URL}/api/v1/plants?page=1&limit=20&name=&status=")

echo "$plants_json" | jq -e '
  (.data.infos | length) > 0 and
  (.data.infos[] | has("id") and has("name") and has("status") and has("address") and has("pac") and has("etoday") and has("etotal") and has("updateAt") and has("plantPermission") and has("existCamera"))
' >/dev/null

plant_id=$(echo "$plants_json" | jq -er '.data.infos[0].id')
echo "[+] Plants response OK; using plant_id=${plant_id}"

echo "[*] Fetching power flow..."
flow_json=$(curl -sS -H "${auth_header[@]}" "${BASE_URL}/api/v1/plant/energy/${plant_id}/flow?date=${date_utc}")

echo "$flow_json" | jq -e '
  (.data | has("pvPower") and has("battPower") and has("gridOrMeterPower") and has("loadOrEpsPower") and has("soc") and has("existsMeter") and has("existsGrid") and has("existsGen") and has("time"))
' >/dev/null

echo "[+] Power flow response OK for ${date_utc}"

for days_ago in 1 7 30; do
  history_date=$(utc_date_days_ago "$days_ago")
  echo "[*] Fetching day energy for ${history_date}..."
  day_json=$(curl -sS -H "${auth_header[@]}" "${BASE_URL}/api/v1/plant/energy/${plant_id}/day?lan=en&date=${history_date}&id=${plant_id}")

  echo "$day_json" | jq -e '
    (.success == true) and
    (.data | type == "object") and
    (.data.infos | type == "array") and
    (.data.infos | length) > 0 and
    ([.data.infos[] | select((.records | type) == "array") | .records[]? | select(has("time") and has("value") and has("updateTime"))] | length) > 0
  ' >/dev/null

  series_count=$(echo "$day_json" | jq '[.data.infos[] | select((.records | type) == "array" and (.records | length) > 0)] | length')
  record_count=$(echo "$day_json" | jq '[.data.infos[]?.records[]? | select(has("time") and has("value") and has("updateTime"))] | length')
  labels=$(echo "$day_json" | jq -r '[.data.infos[] | select((.records | type) == "array" and (.records | length) > 0) | .label] | join(", ")')
  echo "[+] Day energy response OK for ${history_date}: ${series_count} series, ${record_count} records (${labels})"
done

echo "[✓] API smoke test completed successfully."
