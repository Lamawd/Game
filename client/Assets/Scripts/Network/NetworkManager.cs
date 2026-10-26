using System;
using System.Collections.Generic;
using UnityEngine;
using NativeWebSocket;  // NativeWebSocket package

/// <summary>
/// Persistent singleton that owns the WebSocket connection to the Rust backend.
/// Attach to a "NetworkManager" GameObject and mark DontDestroyOnLoad.
/// </summary>
public class NetworkManager : MonoBehaviour
{
    public static NetworkManager Instance { get; private set; }

    [Header("Server Settings")]
    [SerializeField] private string serverHost = "localhost";
    [SerializeField] private int serverPort = 8080;

    // Events other systems subscribe to
    public event Action<ServerMessage> OnMessageReceived;
    public event Action OnConnected;
    public event Action<string> OnDisconnected;

    private WebSocket _socket;
    private bool _isConnecting;

    // ── Unity lifecycle ──────────────────────────────────────────────────────

    private void Awake()
    {
        if (Instance != null && Instance != this)
        {
            Destroy(gameObject);
            return;
        }
        Instance = this;
        DontDestroyOnLoad(gameObject);
    }

    private void Update()
    {
#if !UNITY_WEBGL || UNITY_EDITOR
        _socket?.DispatchMessageQueue();
#endif
    }

    private async void OnDestroy()
    {
        if (_socket != null)
            await _socket.Close();
    }

    // ── Public API ───────────────────────────────────────────────────────────

    /// <summary>Connect this player to a lobby WebSocket session.</summary>
    public async void Connect(string lobbyId, string playerId)
    {
        if (_isConnecting || _socket?.State == WebSocketState.Open)
            return;

        _isConnecting = true;
        string url = $"ws://{serverHost}:{serverPort}/ws/{lobbyId}/{playerId}";
        Debug.Log($"[Network] Connecting to {url}");

        _socket = new WebSocket(url);

        _socket.OnOpen += () =>
        {
            _isConnecting = false;
            Debug.Log("[Network] Connected.");
            OnConnected?.Invoke();
        };

        _socket.OnMessage += (bytes) =>
        {
            string json = System.Text.Encoding.UTF8.GetString(bytes);
            HandleRawMessage(json);
        };

        _socket.OnError += (error) =>
        {
            Debug.LogError($"[Network] Error: {error}");
        };

        _socket.OnClose += (code) =>
        {
            _isConnecting = false;
            Debug.Log($"[Network] Disconnected (code {code}).");
            OnDisconnected?.Invoke(code.ToString());
        };

        await _socket.Connect();
    }

    public async void Disconnect()
    {
        if (_socket != null)
            await _socket.Close();
    }

    /// <summary>Send a typed client message to the server.</summary>
    public async void Send(ClientMessage msg)
    {
        if (_socket?.State != WebSocketState.Open)
        {
            Debug.LogWarning("[Network] Tried to send but socket is not open.");
            return;
        }
        string json = JsonUtility.ToJson(msg);
        await _socket.SendText(json);
    }

    // ── Internal ─────────────────────────────────────────────────────────────

    private void HandleRawMessage(string json)
    {
        try
        {
            // Deserialize the "type" discriminator first
            var envelope = JsonUtility.FromJson<MessageEnvelope>(json);
            var msg = ServerMessageParser.Parse(envelope.type, json);
            if (msg != null)
                OnMessageReceived?.Invoke(msg);
        }
        catch (Exception ex)
        {
            Debug.LogError($"[Network] Failed to parse message: {ex.Message}\nRaw: {json}");
        }
    }
}

[Serializable]
internal class MessageEnvelope
{
    public string type;
}
