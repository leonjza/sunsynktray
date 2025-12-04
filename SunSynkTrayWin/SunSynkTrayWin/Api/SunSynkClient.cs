using System.Net;
using System.Net.Http;
using System.Net.Http.Headers;
using System.Net.Http.Json;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using Org.BouncyCastle.Crypto.Parameters;
using Org.BouncyCastle.Security;

namespace SunSynkTrayWin.Api;

/// <summary>
/// Basic client for the SunSynk API. Handles login by fetching the public key,
/// encrypting the password with RSA (PKCS#1 v1.5), exchanging for tokens, and
/// automatically re-authing once on client errors. Implemented as a singleton
/// to share the token across the app.
/// </summary>
public sealed class SunSynkClient
{
    private const string BaseUrl = "https://api.sunsynk.net";
    private const string Source = "sunsynk";
    private const string ClientId = "csp-web";
    private const string GrantType = "password";
    private readonly HttpClient _httpClient;
    private readonly SemaphoreSlim _authGate = new(1, 1);
    private string? _cachedUsername;
    private string? _cachedPassword;

    public static SunSynkClient Instance { get; } = new();

    public string? AccessToken { get; private set; }

    private SunSynkClient(HttpClient? httpClient = null)
    {
        _httpClient = httpClient ?? new HttpClient();
        if (_httpClient.BaseAddress is null)
        {
            _httpClient.BaseAddress = new Uri(BaseUrl);
        }
    }

    /// <summary>
    /// Performs the login flow, caches the credentials, and stores the access token.
    /// </summary>
    public async Task<TokenData> LoginAsync(string username, string password, IProgress<string>? progress = null, CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(username))
            throw new ArgumentException("Username is required", nameof(username));
        if (string.IsNullOrWhiteSpace(password))
            throw new ArgumentException("Password is required", nameof(password));

        await _authGate.WaitAsync(cancellationToken).ConfigureAwait(false);
        try
        {
            _cachedUsername = username;
            _cachedPassword = password;
            var token = await PerformLoginAsync(username, password, progress, cancellationToken).ConfigureAwait(false);
            AccessToken = token.AccessToken;
            return token;
        }
        finally
        {
            _authGate.Release();
        }
    }

    /// <summary>
    /// Fetches a page of plants for the authenticated user.
    /// </summary>
    public async Task<PlantPage> GetPlantsAsync(int page = 1, int limit = 100, CancellationToken cancellationToken = default)
    {
        var uri = $"/api/v1/plants?page={page}&limit={limit}&name=&status=";
        return await SendApiRequestAsync<PlantPage>(HttpMethod.Get, uri, "plants", cancellationToken).ConfigureAwait(false);
    }

    /// <summary>
    /// Fetches current power flow for a plant on a given UTC date.
    /// </summary>
    public async Task<PowerFlowData> GetPowerFlowAsync(int plantId, DateTime dateUtc, CancellationToken cancellationToken = default)
    {
        var dateParam = dateUtc.ToString("yyyy-MM-dd");
        var uri = $"/api/v1/plant/energy/{plantId}/flow?date={dateParam}";
        return await SendApiRequestAsync<PowerFlowData>(HttpMethod.Get, uri, "power flow", cancellationToken).ConfigureAwait(false);
    }

    /// <summary>
    /// Fetches day energy series (e.g., PV, Battery, Load, Grid) for a plant on a given UTC date.
    /// </summary>
    public async Task<DayEnergyData> GetDayEnergyAsync(int plantId, DateTime dateUtc, CancellationToken cancellationToken = default)
    {
        var dateParam = dateUtc.ToString("yyyy-MM-dd");
        var uri = $"/api/v1/plant/energy/{plantId}/day?lan=en&date={dateParam}&id={plantId}";
        return await SendApiRequestAsync<DayEnergyData>(HttpMethod.Get, uri, "day energy", cancellationToken).ConfigureAwait(false);
    }

    public void ClearCachedAuthentication()
    {
        AccessToken = null;
        _cachedUsername = null;
        _cachedPassword = null;
    }

    private async Task<string> FetchPublicKeyAsync(CancellationToken cancellationToken)
    {
        var nonce = NowMs();
        var sign = ComputeMd5Hex($"nonce={nonce}&source={Source}POWER_VIEW");
        var uri = $"/anonymous/publicKey?nonce={nonce}&source={Source}&sign={sign}";

        var result = await _httpClient.GetFromJsonAsync<PublicKeyResponse>(uri, cancellationToken)
            .ConfigureAwait(false) ?? throw new InvalidOperationException("Empty response from public key endpoint");

        if (string.IsNullOrWhiteSpace(result.Data))
        {
            throw new InvalidOperationException($"Failed to fetch public key: {result.Msg ?? "unknown error"}");
        }

        return result.Data;
    }

    private static string EncryptPassword(string publicKeyBase64, string password)
    {
        var publicKeyBytes = Convert.FromBase64String(publicKeyBase64);
        using var rsa = CreateRsaFromSubjectPublicKeyInfo(publicKeyBytes);
        var encrypted = rsa.Encrypt(Encoding.UTF8.GetBytes(password), RSAEncryptionPadding.Pkcs1);
        return Convert.ToBase64String(encrypted);
    }

    private static bool IsAuthError(HttpStatusCode statusCode) =>
        statusCode == HttpStatusCode.Unauthorized || statusCode == HttpStatusCode.Forbidden;

    private HttpRequestMessage CreateAuthorizedRequest(HttpMethod method, string uri)
    {
        EnsureAccessToken();
        var request = new HttpRequestMessage(method, uri);
        request.Headers.Authorization = new AuthenticationHeaderValue("Bearer", AccessToken);
        return request;
    }

    private void EnsureAccessToken()
    {
        if (string.IsNullOrWhiteSpace(AccessToken))
        {
            throw new InvalidOperationException("Client is not authenticated. Call LoginAsync first.");
        }
    }

    private async Task<TokenData> PerformLoginAsync(string username, string password, IProgress<string>? progress, CancellationToken cancellationToken)
    {
        progress?.Report("Fetching public key...");
        var publicKey = await FetchPublicKeyAsync(cancellationToken).ConfigureAwait(false);
        progress?.Report("Public key received.");

        progress?.Report("Encrypting password...");
        var encryptedPassword = EncryptPassword(publicKey, password);
        var loginNonce = NowMs();
        var publicKeyPrefix = publicKey.Substring(0, Math.Min(10, publicKey.Length));
        var sign = ComputeMd5Hex($"nonce={loginNonce}&source={Source}{publicKeyPrefix}");
        progress?.Report("Password encrypted; building login request.");

        var loginRequest = new LoginRequest
        {
            Sign = sign,
            Nonce = loginNonce,
            Username = username,
            Password = encryptedPassword,
            GrantType = GrantType,
            ClientId = ClientId,
            Source = Source
        };

        progress?.Report("Sending login request...");
        using var response = await _httpClient.PostAsJsonAsync("/oauth/token/new", loginRequest, cancellationToken)
            .ConfigureAwait(false);

        response.EnsureSuccessStatusCode();
        progress?.Report("Parsing login response...");
        var tokenResponse = await response.Content.ReadFromJsonAsync<ApiResponse<TokenData>>(cancellationToken: cancellationToken)
            .ConfigureAwait(false) ?? throw new InvalidOperationException("Empty response from login endpoint");

        if (tokenResponse.Data?.AccessToken is null)
        {
            throw new InvalidOperationException($"Login failed: {tokenResponse.Msg ?? "unknown error"}");
        }

        progress?.Report("Login succeeded.");
        return tokenResponse.Data;
    }

    private async Task<bool> TryReAuthenticateAsync(CancellationToken cancellationToken)
    {
        if (string.IsNullOrWhiteSpace(_cachedUsername) || string.IsNullOrWhiteSpace(_cachedPassword))
        {
            return false;
        }

        await _authGate.WaitAsync(cancellationToken).ConfigureAwait(false);
        try
        {
            var token = await PerformLoginAsync(_cachedUsername, _cachedPassword, progress: null, cancellationToken).ConfigureAwait(false);
            AccessToken = token.AccessToken;
            return true;
        }
        catch
        {
            AccessToken = null;
            return false;
        }
        finally
        {
            _authGate.Release();
        }
    }

    private async Task<HttpResponseMessage> SendWithReauthAsync(Func<HttpRequestMessage> requestFactory, CancellationToken cancellationToken)
    {
        var response = await SendOnceAsync(requestFactory, cancellationToken).ConfigureAwait(false);
        if (response.IsSuccessStatusCode)
        {
            return response;
        }

        if (IsAuthError(response.StatusCode))
        {
            var reauthed = await TryReAuthenticateAsync(cancellationToken).ConfigureAwait(false);
            if (reauthed)
            {
                response.Dispose();
                return await SendOnceAsync(requestFactory, cancellationToken).ConfigureAwait(false);
            }
        }

        return response;
    }

    private async Task<HttpResponseMessage> SendOnceAsync(Func<HttpRequestMessage> requestFactory, CancellationToken cancellationToken)
    {
        using var request = requestFactory();
        return await _httpClient.SendAsync(request, cancellationToken).ConfigureAwait(false);
    }

    private async Task<T> SendApiRequestAsync<T>(HttpMethod method, string uri, string context, CancellationToken cancellationToken)
    {
        using var response = await SendWithReauthAsync(
            () => CreateAuthorizedRequest(method, uri),
            cancellationToken).ConfigureAwait(false);

        var payload = await ReadJsonAsync<ApiResponse<T>>(response, cancellationToken, context).ConfigureAwait(false);
        return payload.Data ?? throw new InvalidOperationException($"{context} not available: {payload.Msg ?? "unknown error"}");
    }

    private static async Task<T> ReadJsonAsync<T>(HttpResponseMessage response, CancellationToken cancellationToken, string context)
    {
        response.EnsureSuccessStatusCode();
        var content = await response.Content.ReadFromJsonAsync<T>(cancellationToken: cancellationToken)
            .ConfigureAwait(false) ?? throw new InvalidOperationException($"Empty response from {context} endpoint");
        return content;
    }

    private static long NowMs() => DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();

    private static string ComputeMd5Hex(string input)
    {
        using var md5 = MD5.Create();
        var hash = md5.ComputeHash(Encoding.UTF8.GetBytes(input));
        var hex = new StringBuilder(hash.Length * 2);
        foreach (var b in hash)
        {
            hex.Append(b.ToString("x2"));
        }
        return hex.ToString();
    }

    private static RSA CreateRsaFromSubjectPublicKeyInfo(byte[] subjectPublicKeyInfo)
    {
        // Parse SubjectPublicKeyInfo (as returned by the API) into RSA parameters using BouncyCastle for net48 compatibility.
        var keyParam = (Org.BouncyCastle.Crypto.Parameters.RsaKeyParameters)Org.BouncyCastle.Security.PublicKeyFactory.CreateKey(subjectPublicKeyInfo);
        var rsaParams = new RSAParameters
        {
            Modulus = keyParam.Modulus.ToByteArrayUnsigned(),
            Exponent = keyParam.Exponent.ToByteArrayUnsigned()
        };
        var rsa = RSA.Create();
        rsa.ImportParameters(rsaParams);
        return rsa;
    }

    private record PublicKeyResponse
    {
        [JsonPropertyName("code")] public int Code { get; init; }
        [JsonPropertyName("msg")] public string? Msg { get; init; }
        [JsonPropertyName("data")] public string? Data { get; init; }
        [JsonPropertyName("success")] public bool Success { get; init; }
    }

    private record LoginRequest
    {
        [JsonPropertyName("sign")] public string? Sign { get; init; }
        [JsonPropertyName("nonce")] public long Nonce { get; init; }
        [JsonPropertyName("username")] public string? Username { get; init; }
        [JsonPropertyName("password")] public string? Password { get; init; }
        [JsonPropertyName("grant_type")] public string? GrantType { get; init; }
        [JsonPropertyName("client_id")] public string? ClientId { get; init; }
        [JsonPropertyName("source")] public string? Source { get; init; }
    }

    private record ApiResponse<T>
    {
        [JsonPropertyName("code")] public int Code { get; init; }
        [JsonPropertyName("msg")] public string? Msg { get; init; }
        [JsonPropertyName("data")] public T? Data { get; init; }
        [JsonPropertyName("success")] public bool Success { get; init; }
    }
}
