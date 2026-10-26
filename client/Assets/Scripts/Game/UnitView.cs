using UnityEngine;

/// <summary>
/// Visual representation of a unit on the board or bench.
/// Handles drag-and-drop interaction.
/// </summary>
public class UnitView : MonoBehaviour
{
    public string UnitId   { get; private set; }
    public string UnitName { get; private set; }
    public int    Cost     { get; private set; }
    public int    Tier     { get; private set; }

    public bool IsOnBoard  { get; private set; }
    public int  BoardCol   { get; private set; }
    public int  BoardRow   { get; private set; }

    [SerializeField] private TMPro.TMP_Text nameLabel;
    [SerializeField] private TMPro.TMP_Text tierLabel;

    private Vector3 _dragOffset;
    private bool    _isDragging;
    private Camera  _cam;
    private BoardManager _board;

    private void Start()
    {
        _cam   = Camera.main;
        _board = FindFirstObjectByType<BoardManager>();
    }

    public void Initialize(UnitDto dto)
    {
        UnitId   = dto.id;
        UnitName = dto.name;
        Cost     = dto.cost;
        Tier     = dto.tier;

        if (nameLabel != null) nameLabel.text = dto.name;
        if (tierLabel != null) tierLabel.text = new string('★', dto.tier);
    }

    public void SetBoardPosition(int col, int row)
    {
        IsOnBoard = true;
        BoardCol  = col;
        BoardRow  = row;
    }

    // ── Drag & Drop (mouse-based, works in editor/PC build) ──────────────────

    private void OnMouseDown()
    {
        _isDragging = true;
        _dragOffset = transform.position - GetMouseWorldPos();
    }

    private void OnMouseDrag()
    {
        if (!_isDragging) return;
        transform.position = GetMouseWorldPos() + _dragOffset + Vector3.up * 0.5f;
    }

    private void OnMouseUp()
    {
        if (!_isDragging) return;
        _isDragging = false;

        // Raycast to find target hex tile
        Ray ray = _cam.ScreenPointToRay(Input.mousePosition);
        if (Physics.Raycast(ray, out RaycastHit hit))
        {
            var tile = hit.collider.GetComponent<HexTile>();
            if (tile != null && _board != null)
            {
                _board.OnUnitDropped(this, tile);
                return;
            }
        }

        // Snap back to original position if no valid target
        SnapBack();
    }

    private Vector3 GetMouseWorldPos()
    {
        Vector3 mouse = Input.mousePosition;
        mouse.z = _cam.WorldToScreenPoint(transform.position).z;
        return _cam.ScreenToWorldPoint(mouse);
    }

    private void SnapBack()
    {
        // TODO: animate back to previous position
    }
}
