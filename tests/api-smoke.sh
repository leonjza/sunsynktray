#!/usr/bin/env bash

set -euo pipefail

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
login_response=$(curl -sS -X POST "${BASE_URL}/oauth/token/new" \
  -H "Content-Type: application/json;charset=UTF-8" \
  -d '{
        "sign": "'"$sign_login"'",
        "nonce": '"$login_nonce"',
        "username": "'"$USERNAME"'",
        "password": "'"$encrypted_pwd_b64"'",
        "grant_type": "'"$GRANT_TYPE"'",
        "client_id": "'"$CLIENT_ID"'",
        "source": "'"$SOURCE"'"
      }')

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

echo "[*] Fetching day energy..."
day_json=$(curl -sS -H "${auth_header[@]}" "${BASE_URL}/api/v1/plant/energy/${plant_id}/day?lan=en&date=${date_utc}&id=${plant_id}")

echo "$day_json" | jq -e '
  (.data.infos | length) > 0 and
  (.data.infos[] | has("unit") and has("label") and (.records | length) >= 0 and (.records[]? | has("time") and has("value") and has("updateTime")))
' >/dev/null

echo "[+] Day energy response OK for ${date_utc}"

rm -f "$pem_file"
echo "[✓] API smoke test completed successfully."
