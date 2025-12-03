using System.Text.Json;
using System.Text.Json.Serialization;

namespace SunSynkTrayWin.Api;

public record TokenData
{
    [JsonPropertyName("access_token")] public string? AccessToken { get; init; }
    [JsonPropertyName("refresh_token")] public string? RefreshToken { get; init; }
    [JsonPropertyName("token_type")] public string? TokenType { get; init; }
    [JsonPropertyName("expires_in")] public int? ExpiresIn { get; init; }
}

public record PlantPage
{
    [JsonPropertyName("pageSize")] public int PageSize { get; init; }
    [JsonPropertyName("pageNumber")] public int PageNumber { get; init; }
    [JsonPropertyName("total")] public int Total { get; init; }
    [JsonPropertyName("infos")] public List<PlantInfo> Infos { get; init; } = new();
}

public record PlantInfo
{
    [JsonPropertyName("id")] public int Id { get; init; }
    [JsonPropertyName("name")] public string? Name { get; init; }
    [JsonPropertyName("thumbUrl")] public string? ThumbUrl { get; init; }
    [JsonPropertyName("status")] public int Status { get; init; }
    [JsonPropertyName("address")] public string? Address { get; init; }
    [JsonPropertyName("pac")] public decimal? Pac { get; init; }
    [JsonPropertyName("efficiency")] public decimal? Efficiency { get; init; }
    [JsonPropertyName("etoday")] public decimal? EnergyToday { get; init; }
    [JsonPropertyName("etotal")] public decimal? EnergyTotal { get; init; }
    [JsonPropertyName("updateAt")] public DateTimeOffset? UpdatedAt { get; init; }
    [JsonPropertyName("createAt")] public DateTimeOffset? CreatedAt { get; init; }
    [JsonPropertyName("type")] public int Type { get; init; }
    [JsonPropertyName("masterId")] public int MasterId { get; init; }
    [JsonPropertyName("share")] public bool Share { get; init; }
    [JsonPropertyName("plantPermission")] public List<string> PlantPermission { get; init; } = new();
    [JsonPropertyName("existCamera")] public bool ExistCamera { get; init; }
    [JsonPropertyName("email")] public string? Email { get; init; }
    [JsonPropertyName("phone")] public string? Phone { get; init; }
    [JsonPropertyName("productWarrantyRegistered")] public int ProductWarrantyRegistered { get; init; }
}

public record PowerFlowData
{
    [JsonPropertyName("custCode")] public int CustCode { get; init; }
    [JsonPropertyName("protocolIdentifier")] public string? ProtocolIdentifier { get; init; }
    [JsonPropertyName("meterCode")] public int MeterCode { get; init; }
    [JsonPropertyName("pvPower")] public int PvPower { get; init; }
    [JsonPropertyName("battPower")] public int BattPower { get; init; }
    [JsonPropertyName("battPower2")] public int BattPower2 { get; init; }
    [JsonPropertyName("gridOrMeterPower")] public int GridOrMeterPower { get; init; }
    [JsonPropertyName("loadOrEpsPower")] public int LoadOrEpsPower { get; init; }
    [JsonPropertyName("genPower")] public int GenPower { get; init; }
    [JsonPropertyName("minPower")] public int MinPower { get; init; }
    [JsonPropertyName("soc")] public double? Soc { get; init; }
    [JsonPropertyName("smartLoadPower")] public int SmartLoadPower { get; init; }
    [JsonPropertyName("upsLoadPower")] public int UpsLoadPower { get; init; }
    [JsonPropertyName("homeLoadPower")] public int HomeLoadPower { get; init; }
    [JsonPropertyName("chargePilePower")] public int ChargePilePower { get; init; }
    [JsonPropertyName("pvTo")] public bool PvTo { get; init; }
    [JsonPropertyName("toLoad")] public bool ToLoad { get; init; }
    [JsonPropertyName("toSmartLoad")] public bool ToSmartLoad { get; init; }
    [JsonPropertyName("toUpsLoad")] public bool ToUpsLoad { get; init; }
    [JsonPropertyName("toHomeLoad")] public bool ToHomeLoad { get; init; }
    [JsonPropertyName("toGrid")] public bool ToGrid { get; init; }
    [JsonPropertyName("toBat")] public bool ToBat { get; init; }
    [JsonPropertyName("batTo")] public bool BatTo { get; init; }
    [JsonPropertyName("gridTo")] public bool GridTo { get; init; }
    [JsonPropertyName("genTo")] public bool GenTo { get; init; }
    [JsonPropertyName("minTo")] public bool MinTo { get; init; }
    [JsonPropertyName("toChargePile")] public bool ToChargePile { get; init; }
    [JsonPropertyName("existsGen")] public bool ExistsGen { get; init; }
    [JsonPropertyName("existsMin")] public bool ExistsMin { get; init; }
    [JsonPropertyName("existsGrid")] public bool ExistsGrid { get; init; }
    [JsonPropertyName("genOn")] public bool GenOn { get; init; }
    [JsonPropertyName("microOn")] public bool MicroOn { get; init; }
    [JsonPropertyName("existsMeter")] public bool ExistsMeter { get; init; }
    // Some deployments return 0/1 instead of bool; keep numeric to avoid parse errors.
    [JsonPropertyName("bmsCommFaultFlag"), JsonConverter(typeof(BoolIntJsonConverter))] public int? BmsCommFaultFlag { get; init; }
    [JsonPropertyName("existsThreeLoad")] public bool ExistsThreeLoad { get; init; }
    [JsonPropertyName("existsSmartLoad")] public bool ExistsSmartLoad { get; init; }
    [JsonPropertyName("existsChargePile")] public bool ExistsChargePile { get; init; }
    [JsonPropertyName("pv")] public object? Pv { get; init; }
    [JsonPropertyName("existThinkPower")] public bool ExistThinkPower { get; init; }
    [JsonPropertyName("time")] public string? Time { get; init; }
}

public record DayEnergyData
{
    [JsonPropertyName("infos")] public List<EnergyInfo> Infos { get; init; } = new();
}

public record EnergyInfo
{
    [JsonPropertyName("unit")] public string? Unit { get; init; }
    [JsonPropertyName("records")] public List<EnergyRecord> Records { get; init; } = new();
    [JsonPropertyName("label")] public string? Label { get; init; }
}

public record EnergyRecord
{
    [JsonPropertyName("time")] public string? Time { get; init; }
    [JsonPropertyName("value")] public string? Value { get; init; }
    [JsonPropertyName("updateTime")] public string? UpdateTime { get; init; }
}

/// <summary>
/// Accepts booleans or integers (or numeric strings) and returns 0/1 or the integer provided.
/// </summary>
public class BoolIntJsonConverter : JsonConverter<int?>
{
    public override int? Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
    {
        switch (reader.TokenType)
        {
            case JsonTokenType.True:
                return 1;
            case JsonTokenType.False:
                return 0;
            case JsonTokenType.Number:
                if (reader.TryGetInt32(out var num))
                {
                    return num;
                }
                break;
            case JsonTokenType.String:
                var str = reader.GetString();
                if (int.TryParse(str, out var parsed))
                {
                    return parsed;
                }
                if (bool.TryParse(str, out var boolVal))
                {
                    return boolVal ? 1 : 0;
                }
                break;
        }

        return null;
    }

    public override void Write(Utf8JsonWriter writer, int? value, JsonSerializerOptions options)
    {
        if (value.HasValue)
        {
            writer.WriteNumberValue(value.Value);
        }
        else
        {
            writer.WriteNullValue();
        }
    }
}
