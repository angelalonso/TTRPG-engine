import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import React, { useEffect, useState } from 'react';
import { fetchCatalog, performAction, buyCar } from '../services/tauriApi';
export const CatalogView = ({ gameState, onStateUpdate }) => {
    const [catalog, setCatalog] = useState(null);
    const [error, setError] = useState(null);
    useEffect(() => {
        fetchCatalog()
            .then(setCatalog)
            .catch((err) => setError(String(err)));
    }, []);
    const formatFreq = (act) => {
        if (act.payout_freq_type === 'once')
            return 'one-time';
        const s = act.payout_freq > 1 ? 's' : '';
        return `every ${act.payout_freq} ${act.payout_freq_unit}${s}`;
    };
    const handleAction = async (actionId) => {
        try {
            setError(null);
            await performAction(actionId);
        }
        catch (err) {
            setError(String(err));
        }
    };
    const handleBuyCar = async (carId) => {
        try {
            setError(null);
            const updatedState = await buyCar(carId);
            onStateUpdate(updatedState);
        }
        catch (err) {
            setError(String(err));
        }
    };
    if (!catalog)
        return _jsx("div", { style: { padding: '1rem', color: '#94a3b8' }, children: "Loading Catalog..." });
    return (_jsxs("div", { style: { padding: '1rem', backgroundColor: '#1e293b', borderRadius: '8px', color: '#f8fafc' }, children: [_jsx("h2", { style: { marginTop: 0, borderBottom: '1px solid #334155', paddingBottom: '0.5rem' }, children: "\u26A1 Available Actions" }), error && _jsx("div", { style: { color: '#ef4444', marginBottom: '1rem' }, children: error }), _jsx("ul", { style: { listStyle: 'none', padding: 0 }, children: catalog.actions.map((act) => {
                    const isActive = gameState.player.active_actions?.some((a) => a.action_id === act.id);
                    return (_jsxs("li", { style: {
                            marginBottom: '0.75rem',
                            padding: '0.75rem',
                            backgroundColor: '#0f172a',
                            borderRadius: '6px',
                            border: '1px solid #334155',
                            display: 'flex',
                            justifyContent: 'space-between',
                            alignItems: 'center',
                        }, children: [_jsxs("div", { children: [_jsx("strong", { children: act.name }), " \u2014 Cost: \u00A3", act.base_cost, " | Success Rate:", ' ', (act.success_rate * 100).toFixed(0), "% | Payout: \u00A3", act.payout.toLocaleString(), " (", formatFreq(act), ")"] }), _jsx("button", { onClick: () => handleAction(act.id), disabled: gameState.player.budget < act.base_cost || isActive, style: {
                                    padding: '0.4rem 0.8rem',
                                    backgroundColor: isActive
                                        ? '#064e3b'
                                        : gameState.player.budget >= act.base_cost
                                            ? '#2563eb'
                                            : '#475569',
                                    color: isActive ? '#86efac' : '#ffffff',
                                    border: 'none',
                                    borderRadius: '4px',
                                    cursor: gameState.player.budget >= act.base_cost && !isActive ? 'pointer' : 'not-allowed',
                                    fontWeight: 'bold',
                                }, children: isActive ? 'Active' : 'Execute' })] }, act.id));
                }) }), _jsx("h2", { style: { marginTop: '2rem', borderBottom: '1px solid #334155', paddingBottom: '0.5rem' }, children: "\uD83D\uDE98 Dealership" }), _jsx("div", { style: { display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '1rem' }, children: catalog.cars.map((car) => (_jsxs("div", { style: {
                        padding: '1rem',
                        backgroundColor: '#0f172a',
                        borderRadius: '6px',
                        border: '1px solid #334155',
                        display: 'flex',
                        flexDirection: 'column',
                        justifyContent: 'space-between',
                    }, children: [_jsxs("div", { children: [_jsx("h3", { style: { margin: '0 0 0.5rem 0', color: '#f59e0b' }, children: car.name }), _jsxs("div", { style: { fontSize: '1.25rem', fontWeight: 'bold', color: '#22c55e', marginBottom: '1rem' }, children: ["\u00A3", car.price.toLocaleString()] }), _jsxs("div", { style: { fontSize: '0.85rem', color: '#94a3b8', lineHeight: '1.6', marginBottom: '1rem' }, children: [_jsxs("div", { children: ["Engine Rebuild: \u00A3", car.engine_rebuild_cost.toLocaleString()] }), _jsxs("div", { children: ["Gearbox Maintenance: \u00A3", car.gearbox_maint_cost.toLocaleString()] }), _jsxs("div", { children: ["Oil Change: \u00A3", car.oil_change_cost.toLocaleString()] }), _jsxs("div", { children: ["Tire Set: \u00A3", car.tire_set_cost.toLocaleString()] })] })] }), _jsx("button", { onClick: () => handleBuyCar(car.id), disabled: gameState.player.budget < car.price, style: {
                                width: '100%',
                                padding: '0.5rem',
                                backgroundColor: gameState.player.budget >= car.price ? '#16a34a' : '#475569',
                                color: '#ffffff',
                                border: 'none',
                                borderRadius: '4px',
                                cursor: gameState.player.budget >= car.price ? 'pointer' : 'not-allowed',
                                fontWeight: 'bold',
                            }, children: gameState.player.budget >= car.price ? 'Buy Car' : 'Insufficient Funds' })] }, car.id))) })] }));
};
//# sourceMappingURL=CatalogView.js.map