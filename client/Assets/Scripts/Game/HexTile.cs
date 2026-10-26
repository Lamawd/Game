using UnityEngine;

/// <summary>
/// Represents a single hex tile on the board.
/// Handles visual hover state and drop target detection.
/// </summary>
public class HexTile : MonoBehaviour
{
    public int Col { get; private set; }
    public int Row { get; private set; }

    [SerializeField] private Material defaultMaterial;
    [SerializeField] private Material hoverMaterial;

    private MeshRenderer _renderer;

    public void Initialize(int col, int row)
    {
        Col = col;
        Row = row;
        _renderer = GetComponent<MeshRenderer>();
    }

    public void SetHover(bool hovered)
    {
        if (_renderer == null) return;
        _renderer.material = hovered ? hoverMaterial : defaultMaterial;
    }
}
