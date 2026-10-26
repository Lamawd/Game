using System;
using System.Collections.Generic;
using UnityEngine;

/// <summary>
/// Client-side representation of the local player's state.
/// Kept in sync by messages from the server.
/// </summary>
public class PlayerState : MonoBehaviour
{
    public static PlayerState Local { get; private set; }

    public string PlayerId   { get; private set; }
    public string Username   { get; private set; }
    public int    Gold       { get; private set; }
    public int    Health     { get; private set; }
    public int    Level      { get; private set; }

    public event Action OnStateChanged;

    private void Awake()
    {
        if (Local != null && Local != this) { Destroy(gameObject); return; }
        Local = this;
        DontDestroyOnLoad(gameObject);
    }

    private void Start()
    {
        if (NetworkManager.Instance != null)
            NetworkManager.Instance.OnMessageReceived += HandleMessage;
    }

    private void OnDestroy()
    {
        if (NetworkManager.Instance != null)
            NetworkManager.Instance.OnMessageReceived -= HandleMessage;
    }

    public void Initialize(string playerId, string username)
    {
        PlayerId = playerId;
        Username = username;
    }

    private void HandleMessage(ServerMessage msg)
    {
        switch (msg)
        {
            case PlayerUpdateMessage update when update.player_id == PlayerId:
                Gold   = update.gold;
                Health = update.health;
                Level  = update.level;
                OnStateChanged?.Invoke();
                break;

            case GameStateMessage state:
                foreach (var p in state.players)
                {
                    if (p.player_id == PlayerId)
                    {
                        Gold   = p.gold;
                        Health = p.health;
                        Level  = p.level;
                        OnStateChanged?.Invoke();
                        break;
                    }
                }
                break;
        }
    }

    // ── Player Actions (send to server) ─────────────────────────────────────

    public void RollShop()   => NetworkManager.Instance.Send(new RollShopMessage());
    public void BuyXp()      => NetworkManager.Instance.Send(new BuyXpMessage());
    public void Ready()      => NetworkManager.Instance.Send(new ReadyMessage());

    public void BuyUnit(string unitName, int cost)
        => NetworkManager.Instance.Send(new BuyUnitMessage { unit_name = unitName, cost = cost });

    public void SellUnit(string unitId)
        => NetworkManager.Instance.Send(new SellUnitMessage { unit_id = unitId });
}
