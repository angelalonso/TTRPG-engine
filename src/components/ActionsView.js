import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import React, { useState } from 'react';
import { performAction, buyCar, getGameState } from '../services/tauriApi';
export const ActionsView = ({ gameState, onStateUpdate, }) => {
    const { player, catalog } = gameState;
    const [actionResult, setActionResult] = useState(null);
    const [error, setError] = useState(null);
    const [loadingActionId, setLoadingActionId] = useState(null);
    const [loadingCarId, setLoadingCarId] = useState(null);
    const formatFreq = (action) => {
        if (action.payout_freq_type === 'once')
            return 'One-time';
        const s = action.payout_freq > 1 ? 's' : '';
        return `Every ${action.payout_freq} ${action.payout_freq_unit}${s}`;
    };
    const handlePerformAction = async (actionId) => {
        try {
            setError(null);
            setLoadingActionId(actionId);
            const result = await performAction(actionId);
            setActionResult(result);
            const updatedState = await getGameState();
            onStateUpdate(updatedState);
        }
        catch (err) {
            setError(String(err));
        }
        finally {
            setLoadingActionId(null);
        }
    };
    const handleBuyCar = async (carId) => {
        try {
            setError(null);
            setLoadingCarId(carId);
            const updatedState = await buyCar(carId);
            onStateUpdate(updatedState);
            setActionResult(null);
        }
        catch (err) {
            setError(String(err));
        }
        finally {
            setLoadingCarId(null);
        }
    };
    return (_jsxs("div", { style: styles.container, children: [error && _jsx("div", { style: styles.errorBox, children: error }), actionResult && (_jsxs("div", { style: {
                    ...styles.resultBox,
                    borderColor: actionResult.success ? '#22c55e' : '#ef4444',
                }, children: [_jsxs("div", { style: styles.resultHeader, children: [_jsxs("h3", { style: { margin: 0 }, children: [actionResult.success ? '✅ Success:' : '❌ Failed:', ' ', actionResult.action_name] }), _jsx("button", { onClick: () => setActionResult(null), style: styles.closeBtn, children: "\u2715" })] }), _jsx("p", { style: { margin: '0.5rem 0' }, children: actionResult.message }), _jsxs("div", { style: styles.resultFinancials, children: [actionResult.cost_paid > 0 && (_jsxs("span", { style: { color: '#ef4444' }, children: ["Cost: -\u00A3", actionResult.cost_paid.toLocaleString()] })), actionResult.payout_received > 0 && (_jsxs("span", { style: { color: '#22c55e' }, children: ["Payout: +\u00A3", actionResult.payout_received.toLocaleString()] }))] })] })), _jsxs("section", { style: styles.section, children: [_jsx("h2", { style: { margin: 0 }, children: "\u26A1 Daily Jobs & Side Activities" }), _jsx("p", { style: { color: '#94a3b8', fontSize: '0.85rem' }, children: "Perform work or side jobs to earn regular income." }), _jsx("div", { style: styles.grid, children: catalog.actions.map((action) => {
                            const canAfford = player.budget >= action.base_cost;
                            const isLoading = loadingActionId === action.id;
                            const isActive = player.active_actions?.some((a) => a.action_id === action.id);
                            return (_jsxs("div", { style: styles.card, children: [_jsxs("div", { style: styles.cardHeader, children: [_jsx("h3", { style: { margin: 0 }, children: action.name }), _jsx("span", { style: styles.typeBadge, children: action.type })] }), _jsxs("div", { style: styles.statsRow, children: [_jsxs("div", { children: [_jsx("span", { style: styles.label, children: "Base Cost:" }), _jsxs("div", { children: ["\u00A3", action.base_cost.toLocaleString()] })] }), _jsxs("div", { children: [_jsxs("span", { style: styles.label, children: ["Payout (", formatFreq(action), "):"] }), _jsxs("div", { style: { color: '#22c55e', fontWeight: 'bold' }, children: ["+\u00A3", action.payout.toLocaleString()] })] }), _jsxs("div", { children: [_jsx("span", { style: styles.label, children: "Success Rate:" }), _jsxs("div", { children: [(action.success_rate * 100).toFixed(0), "%"] })] }), _jsxs("div", { children: [_jsx("span", { style: styles.label, children: "Risk Factor:" }), _jsxs("div", { children: [(action.risk_factor * 100).toFixed(0), "%"] })] })] }), _jsx("button", { onClick: () => handlePerformAction(action.id), disabled: !canAfford || isLoading || isActive, style: isActive
                                            ? styles.activeBtn
                                            : canAfford && !isLoading
                                                ? styles.primaryBtn
                                                : styles.disabledBtn, children: isLoading
                                            ? 'Executing...'
                                            : isActive
                                                ? 'Active (Started)'
                                                : `Start ${action.name}` })] }, action.id));
                        }) })] }), _jsxs("section", { style: styles.section, children: [_jsx("h2", { style: { margin: 0 }, children: "\uD83C\uDFCE\uFE0F Dealership Catalog" }), _jsx("p", { style: { color: '#94a3b8', fontSize: '0.85rem' }, children: "Purchase cars to build your garage racing fleet." }), _jsx("div", { style: styles.grid, children: catalog.cars.map((car) => {
                            const canAfford = player.budget >= car.price;
                            const ownedCount = player.cars.filter((c) => c.name === car.name).length;
                            const isLoading = loadingCarId === car.id;
                            return (_jsxs("div", { style: styles.card, children: [_jsxs("div", { style: styles.cardHeader, children: [_jsx("h3", { style: { margin: 0 }, children: car.name }), ownedCount > 0 && (_jsxs("span", { style: styles.ownedBadge, children: ["Owned: ", ownedCount] }))] }), _jsxs("div", { style: styles.priceTag, children: ["\u00A3", car.price.toLocaleString()] }), _jsxs("div", { style: styles.maintPreview, children: [_jsx("span", { style: styles.label, children: "Maintenance Costs:" }), _jsxs("div", { style: styles.maintGrid, children: [_jsxs("span", { children: ["Oil Change: \u00A3", car.oil_change_cost] }), _jsxs("span", { children: ["Tire Set: \u00A3", car.tire_set_cost] }), _jsxs("span", { children: ["Engine Rebuild: \u00A3", car.engine_rebuild_cost] }), _jsxs("span", { children: ["Gearbox Service: \u00A3", car.gearbox_maint_cost] })] })] }), _jsx("button", { onClick: () => handleBuyCar(car.id), disabled: !canAfford || isLoading, style: canAfford && !isLoading
                                            ? styles.buyBtn
                                            : styles.disabledBtn, children: isLoading
                                            ? 'Purchasing...'
                                            : `Buy Vehicle (£${car.price.toLocaleString()})` })] }, car.id));
                        }) })] })] }));
};
const styles = {
    container: {
        display: 'flex',
        flexDirection: 'column',
        gap: '1.5rem',
    },
    errorBox: {
        padding: '0.75rem',
        backgroundColor: '#7f1d1d',
        color: '#fca5a5',
        borderRadius: '4px',
        fontSize: '0.9rem',
    },
    resultBox: {
        padding: '1rem',
        backgroundColor: '#0f172a',
        border: '2px solid',
        borderRadius: '6px',
    },
    resultHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
    },
    closeBtn: {
        background: 'none',
        border: 'none',
        color: '#94a3b8',
        fontSize: '1.2rem',
        cursor: 'pointer',
    },
    resultFinancials: {
        display: 'flex',
        gap: '1rem',
        fontSize: '0.9rem',
        marginTop: '0.5rem',
        fontWeight: 'bold',
    },
    section: {
        backgroundColor: '#1e293b',
        padding: '1rem',
        borderRadius: '8px',
    },
    grid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
        gap: '1rem',
        marginTop: '1rem',
    },
    card: {
        backgroundColor: '#0f172a',
        border: '1px solid #334155',
        borderRadius: '6px',
        padding: '1rem',
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'space-between',
        gap: '0.75rem',
    },
    cardHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
    },
    typeBadge: {
        fontSize: '0.75rem',
        backgroundColor: '#334155',
        color: '#38bdf8',
        padding: '0.2rem 0.5rem',
        borderRadius: '4px',
        fontWeight: 'bold',
    },
    ownedBadge: {
        fontSize: '0.75rem',
        backgroundColor: '#15803d',
        color: '#ffffff',
        padding: '0.2rem 0.5rem',
        borderRadius: '4px',
        fontWeight: 'bold',
    },
    priceTag: {
        fontSize: '1.4rem',
        fontWeight: 'bold',
        color: '#38bdf8',
    },
    statsRow: {
        display: 'grid',
        gridTemplateColumns: '1fr 1fr',
        gap: '0.5rem',
        fontSize: '0.85rem',
        backgroundColor: '#1e293b',
        padding: '0.5rem',
        borderRadius: '4px',
    },
    label: {
        fontSize: '0.75rem',
        color: '#94a3b8',
        display: 'block',
    },
    maintPreview: {
        fontSize: '0.8rem',
        backgroundColor: '#1e293b',
        padding: '0.5rem',
        borderRadius: '4px',
    },
    maintGrid: {
        display: 'grid',
        gridTemplateColumns: '1fr 1fr',
        gap: '0.25rem',
        color: '#cbd5e1',
        marginTop: '0.25rem',
    },
    primaryBtn: {
        padding: '0.5rem 0.75rem',
        borderRadius: '4px',
        border: 'none',
        backgroundColor: '#2563eb',
        color: '#ffffff',
        fontWeight: 'bold',
        cursor: 'pointer',
    },
    activeBtn: {
        padding: '0.5rem 0.75rem',
        borderRadius: '4px',
        border: '1px solid #16a34a',
        backgroundColor: '#064e3b',
        color: '#86efac',
        fontWeight: 'bold',
        cursor: 'default',
    },
    buyBtn: {
        padding: '0.5rem 0.75rem',
        borderRadius: '4px',
        border: 'none',
        backgroundColor: '#16a34a',
        color: '#ffffff',
        fontWeight: 'bold',
        cursor: 'pointer',
    },
    disabledBtn: {
        padding: '0.5rem 0.75rem',
        borderRadius: '4px',
        border: '1px solid #475569',
        backgroundColor: '#334155',
        color: '#64748b',
        cursor: 'not-allowed',
        fontWeight: 'bold',
    },
};
//# sourceMappingURL=ActionsView.js.map