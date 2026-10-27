using System;
using UnityEngine;

// ── Messages: CLIENT → SERVER ────────────────────────────────────────────────

/// Base class; "type" field is the discriminator the Rust server reads.
[Serializable]
public abstract class ClientMessage
{
    public string type;
}

[Serializable] public class ReadyMessage        : ClientMessage { public ReadyMessage()        { type = "ready"; } }
[Serializable] public class PingMessage         : ClientMessage { public PingMessage()         { type = "ping"; } }
[Serializable] public class RollShopMessage     : ClientMessage { public RollShopMessage()     { type = "roll_shop"; } }
[Serializable] public class BuyXpMessage        : ClientMessage { public BuyXpMessage()        { type = "buy_xp"; } }

[Serializable]
public class BuyUnitMessage : ClientMessage
{
    public string unit_name;
    public int cost;
    public BuyUnitMessage() { type = "buy_unit"; }
}

[Serializable]
public class SellUnitMessage : ClientMessage
{
    public string unit_id;
    public SellUnitMessage() { type = "sell_unit"; }
}

[Serializable]
public class PlaceUnitMessage : ClientMessage
{
    public string unit_id;
    public HexPositionDto position;
    public PlaceUnitMessage() { type = "place_unit"; }
}

[Serializable]
public class MoveUnitMessage : ClientMessage
{
    public HexPositionDto from;
    public HexPositionDto to;
    public MoveUnitMessage() { type = "move_unit"; }
}

[Serializable]
public class BenchUnitMessage : ClientMessage
{
    public HexPositionDto position;
    public BenchUnitMessage() { type = "bench_unit"; }
}

// ── Messages: SERVER → CLIENT ────────────────────────────────────────────────

[Serializable]
public abstract class ServerMessage
{
    public string type;
}

[Serializable]
public class ConnectedMessage : ServerMessage
{
    public string player_id;
    public string lobby_id;
}

[Serializable]
public class GameStateMessage : ServerMessage
{
    public int round;
    public string phase;
    public PlayerSnapshot[] players;
}

[Serializable]
public class ShopUpdateMessage : ServerMessage
{
    public UnitDto[] units;
}

[Serializable]
public class BoardStateMessage : ServerMessage
{
    public string player_id;
    /// Serialized as Dictionary — Unity JsonUtility không support Dictionary,
    /// nên server gửi dưới dạng array của BoardCell để dễ parse.
    public BoardCell[] placed;
    public UnitDto[] bench;
    public int placed_count;
}

[Serializable]
public class BoardCell
{
    public string position; // "col,row"
    public UnitDto unit;
}
{
    public string winner_player_id; // null = draw
    public int damage_dealt;
    public int surviving_unit_count;
}

[Serializable]
public class PlayerUpdateMessage : ServerMessage
{
    public string player_id;
    public int gold;
    public int health;
    public int level;
}

[Serializable]
public class ErrorMessage : ServerMessage
{
    public string message;
}

[Serializable]
public class PongMessage : ServerMessage { }

// ── Shared DTOs ──────────────────────────────────────────────────────────────

[Serializable]
public class HexPositionDto
{
    public int col;
    public int row;
}

[Serializable]
public class PlayerSnapshot
{
    public string player_id;
    public string username;
    public int health;
    public int gold;
    public int level;
}

[Serializable]
public class UnitDto
{
    public string id;
    public string name;
    public int cost;
    public int tier;
    public string[] traits;
    public int base_hp;
    public int base_attack;
}
