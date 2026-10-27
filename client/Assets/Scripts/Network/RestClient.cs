using System.Collections;
using System.Net.Http;
using System.Text;
using System.Threading.Tasks;
using UnityEngine;
using UnityEngine.Networking;

/// <summary>
/// REST client for pre-game actions: create/join lobby, register player.
/// Uses UnityWebRequest so it works on all Unity platforms.
/// </summary>
public class RestClient : MonoBehaviour
{
    public static RestClient Instance { get; private set; }

    [SerializeField] private string serverHost = "localhost";
    [SerializeField] private int serverPort = 8080;

    private string BaseUrl => $"http://{serverHost}:{serverPort}/api/v1";

    private void Awake()
    {
        if (Instance != null && Instance != this) { Destroy(gameObject); return; }
        Instance = this;
        DontDestroyOnLoad(gameObject);
    }

    // ── Public Helpers ───────────────────────────────────────────────────────

    public void RegisterPlayer(string username, System.Action<PlayerResponse> onSuccess, System.Action<string> onError)
    {
        var body = JsonUtility.ToJson(new RegisterPlayerRequest { username = username });
        StartCoroutine(Post("/player/register", body, onSuccess, onError));
    }

    public void CreateLobby(string hostPlayerId, int maxPlayers, System.Action<LobbyResponse> onSuccess, System.Action<string> onError)
    {
        var body = JsonUtility.ToJson(new CreateLobbyRequest
        {
            host_player_id = hostPlayerId,
            max_players = maxPlayers
        });
        StartCoroutine(Post("/lobby/create", body, onSuccess, onError));
    }

    public void JoinLobby(string lobbyId, string playerId, System.Action<LobbyResponse> onSuccess, System.Action<string> onError)
    {
        var body = JsonUtility.ToJson(new JoinLobbyRequest { player_id = playerId });
        StartCoroutine(Post($"/lobby/{lobbyId}/join", body, onSuccess, onError));
    }

    /// <summary>
    /// Điền bots vào slot còn trống và bắt đầu game loop.
    /// Gọi khi muốn test: 1 người thật + bots.
    /// </summary>
    public void StartLobby(string lobbyId, System.Action<StartLobbyResponse> onSuccess, System.Action<string> onError)
    {
        StartCoroutine(Post($"/lobby/{lobbyId}/start", "{}", onSuccess, onError));
    }

    /// <summary>
    /// Điền bots vào slot còn trống và bắt đầu game loop.
    /// Gọi khi muốn test: 1 người thật + bots.
    /// </summary>
    public void StartLobby(string lobbyId, System.Action<StartLobbyResponse> onSuccess, System.Action<string> onError)
    {
        StartCoroutine(Post($"/lobby/{lobbyId}/start", "{}", onSuccess, onError));
    }

    // ── Internal ─────────────────────────────────────────────────────────────

    private IEnumerator Post<T>(string path, string jsonBody, System.Action<T> onSuccess, System.Action<string> onError)
    {
        string url = BaseUrl + path;
        using var req = new UnityWebRequest(url, "POST");
        byte[] bodyBytes = Encoding.UTF8.GetBytes(jsonBody);
        req.uploadHandler = new UploadHandlerRaw(bodyBytes);
        req.downloadHandler = new DownloadHandlerBuffer();
        req.SetRequestHeader("Content-Type", "application/json");

        yield return req.SendWebRequest();

        if (req.result != UnityWebRequest.Result.Success)
        {
            Debug.LogError($"[REST] POST {path} failed: {req.error}");
            onError?.Invoke(req.error);
        }
        else
        {
            var result = JsonUtility.FromJson<T>(req.downloadHandler.text);
            onSuccess?.Invoke(result);
        }
    }
}

// ── Request / Response DTOs ──────────────────────────────────────────────────

[System.Serializable] public class RegisterPlayerRequest { public string username; }
[System.Serializable] public class CreateLobbyRequest   { public string host_player_id; public int max_players; }
[System.Serializable] public class JoinLobbyRequest     { public string player_id; }

[System.Serializable]
public class PlayerResponse
{
    public PlayerDto player;
}

[System.Serializable]
public class PlayerDto
{
    public string id;
    public string username;
    public int gold;
    public int health;
    public int level;
}

[System.Serializable]
public class LobbyResponse
{
    public LobbyDto lobby;
}

[System.Serializable]
public class LobbyDto
{
    public string id;
    public string host_player_id;
    public string[] player_ids;
    public int max_players;
    public string status;
    public int round;
}

[System.Serializable]
public class StartLobbyResponse
{
    public string lobby_id;
    public int human_count;
    public int bot_count;
    public string[] bot_names;
}
