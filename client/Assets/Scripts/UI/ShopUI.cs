using System.Collections.Generic;
using UnityEngine;
using UnityEngine.UI;
using TMPro;

/// <summary>
/// Manages the 5-unit shop UI panel.
/// Displays offered units and handles buy button clicks.
/// </summary>
public class ShopUI : MonoBehaviour
{
    [SerializeField] private Button   rollButton;
    [SerializeField] private Button   buyXpButton;
    [SerializeField] private TMP_Text goldLabel;
    [SerializeField] private TMP_Text levelLabel;

    [SerializeField] private ShopSlotUI[] slots; // 5 slots

    private void Start()
    {
        rollButton?.onClick.AddListener(() => PlayerState.Local.RollShop());
        buyXpButton?.onClick.AddListener(() => PlayerState.Local.BuyXp());

        if (NetworkManager.Instance != null)
            NetworkManager.Instance.OnMessageReceived += HandleMessage;

        if (PlayerState.Local != null)
            PlayerState.Local.OnStateChanged += RefreshPlayerStats;
    }

    private void OnDestroy()
    {
        if (NetworkManager.Instance != null)
            NetworkManager.Instance.OnMessageReceived -= HandleMessage;

        if (PlayerState.Local != null)
            PlayerState.Local.OnStateChanged -= RefreshPlayerStats;
    }

    private void HandleMessage(ServerMessage msg)
    {
        if (msg is ShopUpdateMessage shopMsg)
            RefreshShop(shopMsg.units);
    }

    private void RefreshShop(UnitDto[] units)
    {
        for (int i = 0; i < slots.Length; i++)
        {
            if (i < units.Length)
                slots[i].SetUnit(units[i]);
            else
                slots[i].SetEmpty();
        }
    }

    private void RefreshPlayerStats()
    {
        var p = PlayerState.Local;
        if (p == null) return;
        if (goldLabel  != null) goldLabel.text  = $"Gold: {p.Gold}";
        if (levelLabel != null) levelLabel.text = $"Lv {p.Level}";
    }
}
