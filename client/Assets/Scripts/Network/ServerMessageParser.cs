using UnityEngine;

/// <summary>
/// Parses raw JSON into the correct ServerMessage subclass by reading the "type" field.
/// Add new message types here as the server grows.
/// </summary>
public static class ServerMessageParser
{
    public static ServerMessage Parse(string type, string json)
    {
        return type switch
        {
            "connected"      => JsonUtility.FromJson<ConnectedMessage>(json),
            "game_state"     => JsonUtility.FromJson<GameStateMessage>(json),
            "shop_update"    => JsonUtility.FromJson<ShopUpdateMessage>(json),
            "board_state"    => JsonUtility.FromJson<BoardStateMessage>(json),
            "combat_result"  => JsonUtility.FromJson<CombatResultMessage>(json),
            "player_update"  => JsonUtility.FromJson<PlayerUpdateMessage>(json),
            "error"          => JsonUtility.FromJson<ErrorMessage>(json),
            "pong"           => JsonUtility.FromJson<PongMessage>(json),
            _                => null,
        };
    }
}
