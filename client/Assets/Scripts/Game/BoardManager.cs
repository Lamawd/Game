using System.Collections.Generic;
using UnityEngine;

/// <summary>
/// Manages the hex grid board.
/// Generates hex tiles, tracks unit placements, and handles drag-and-drop.
/// </summary>
public class BoardManager : MonoBehaviour
{
    [Header("Grid Settings")]
    [SerializeField] private int columns = 7;
    [SerializeField] private int rows    = 4;

    [Header("Prefabs")]
    [SerializeField] private GameObject hexTilePrefab;
    [SerializeField] private GameObject unitPrefab;

    [Header("Hex Spacing")]
    [SerializeField] private float hexWidth  = 1.1f;
    [SerializeField] private float hexHeight = 0.95f;

    private HexTile[,] _grid;
    private Dictionary<string, UnitView> _placedUnits = new();

    private void Start()
    {
        GenerateGrid();

        if (NetworkManager.Instance != null)
            NetworkManager.Instance.OnMessageReceived += HandleMessage;
    }

    private void OnDestroy()
    {
        if (NetworkManager.Instance != null)
            NetworkManager.Instance.OnMessageReceived -= HandleMessage;
    }

    // ── Grid Generation ──────────────────────────────────────────────────────

    private void GenerateGrid()
    {
        _grid = new HexTile[columns, rows];
        for (int col = 0; col < columns; col++)
        {
            for (int row = 0; row < rows; row++)
            {
                Vector3 pos = HexToWorld(col, row);
                var tile = Instantiate(hexTilePrefab, pos, Quaternion.identity, transform);
                tile.name = $"Tile_{col}_{row}";
                var hexTile = tile.GetComponent<HexTile>();
                hexTile.Initialize(col, row);
                _grid[col, row] = hexTile;
            }
        }
    }

    private Vector3 HexToWorld(int col, int row)
    {
        float xOffset = (row % 2 == 0) ? 0f : hexWidth * 0.5f;
        float x = col * hexWidth + xOffset;
        float z = row * hexHeight;
        return new Vector3(x, 0, z);
    }

    // ── Message Handling ─────────────────────────────────────────────────────

    private void HandleMessage(ServerMessage msg)
    {
        if (msg is BoardUpdate update)
        {
            // Full board sync would come from a GameStateMessage
            // For now, just log — request a full state refresh
            Debug.Log($"[Board] Board updated for player {update.player_id}");
        }
    }

    // ── Drag & Drop ──────────────────────────────────────────────────────────

    /// Called by UnitView when dropped on a tile
    public void OnUnitDropped(UnitView unit, HexTile targetTile)
    {
        int col = targetTile.Col;
        int row = targetTile.Row;

        if (unit.IsOnBoard)
        {
            // Move from one tile to another
            NetworkManager.Instance.Send(new MoveUnitMessage
            {
                from = new HexPositionDto { col = unit.BoardCol, row = unit.BoardRow },
                to   = new HexPositionDto { col = col, row = row }
            });
        }
        else
        {
            // Place from bench
            NetworkManager.Instance.Send(new PlaceUnitMessage
            {
                unit_id  = unit.UnitId,
                position = new HexPositionDto { col = col, row = row }
            });
        }

        // Optimistically update local position
        unit.SetBoardPosition(col, row);
    }
}
