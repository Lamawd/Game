using UnityEngine;
using UnityEngine.UI;
using TMPro;

/// <summary>
/// One slot in the shop panel showing a unit offer.
/// </summary>
public class ShopSlotUI : MonoBehaviour
{
    [SerializeField] private TMP_Text nameLabel;
    [SerializeField] private TMP_Text costLabel;
    [SerializeField] private TMP_Text traitsLabel;
    [SerializeField] private Button   buyButton;
    [SerializeField] private GameObject emptyOverlay;

    private UnitDto _currentUnit;

    private void Start()
    {
        buyButton?.onClick.AddListener(OnBuyClicked);
    }

    public void SetUnit(UnitDto unit)
    {
        _currentUnit = unit;
        if (nameLabel   != null) nameLabel.text   = unit.name;
        if (costLabel   != null) costLabel.text   = $"{unit.cost}g";
        if (traitsLabel != null) traitsLabel.text = string.Join(", ", unit.traits ?? new string[0]);
        if (emptyOverlay != null) emptyOverlay.SetActive(false);
        if (buyButton    != null) buyButton.interactable = true;
    }

    public void SetEmpty()
    {
        _currentUnit = null;
        if (nameLabel    != null) nameLabel.text = "";
        if (costLabel    != null) costLabel.text = "";
        if (traitsLabel  != null) traitsLabel.text = "";
        if (emptyOverlay != null) emptyOverlay.SetActive(true);
        if (buyButton    != null) buyButton.interactable = false;
    }

    private void OnBuyClicked()
    {
        if (_currentUnit == null) return;
        PlayerState.Local.BuyUnit(_currentUnit.name, _currentUnit.cost);
        SetEmpty(); // optimistic clear; server will re-sync
    }
}
