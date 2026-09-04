import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import React, { useState } from 'react';
import { maintainCar } from '../services/tauriApi';
export const Garage = ({ gameState, onStateUpdate }) => {
    const { player } = gameState;
    const [error, setError] = useState(null);
    const [tireCounts, setTireCounts] = useState({});
    const handleMaintenance = async (carId, maintType) => {
        try {
            setError(null);
            const updatedState = await maintainCar(carId, maintType);
            onStateUpdate(updatedState);
        }
        catch (err) {
            setError(String(err));
        }
    };
    const getTireCount = (carId) => tireCounts[carId] ?? 4; // Default to 4 sets (1 race worth)
    const handleTireCountChange = (carId, count) => {
        setTireCounts((prev) => ({ ...prev, [carId]: Math.max(1, count) }));
    };
    if (player.cars.length === 0) {
        return (_jsxs("div", { style: styles.card, children: [_jsx("h2", { children: "\uD83D\uDE97 Garage" }), _jsx("p", { style: { color: '#94a3b8' }, children: "You don't own any cars yet. Purchase one from the Dealership to start racing!" })] }));
    }
    return (_jsxs("div", { style: styles.card, children: [_jsx("h2", { children: "\uD83D\uDE97 Garage & Maintenance" }), error && _jsx("div", { style: styles.errorBox, children: error }), _jsx("div", { style: styles.carGrid, children: player.cars.map((car) => {
                    const selectedTires = getTireCount(car.id);
                    const totalTireCost = car.tire_set_cost * selectedTires;
                    return (_jsxs("div", { style: styles.carCard, children: [_jsxs("div", { style: styles.carHeader, children: [_jsx("h3", { style: { margin: 0 }, children: car.name }), _jsx("span", { style: car.tire_sets_available >= 4 &&
                                            !car.needs_oil_change &&
                                            !car.needs_engine_rebuild &&
                                            !car.needs_gearbox_maint
                                            ? styles.badgeReady
                                            : styles.badgeNotReady, children: car.tire_sets_available >= 4 &&
                                            !car.needs_oil_change &&
                                            !car.needs_engine_rebuild &&
                                            !car.needs_gearbox_maint
                                            ? 'Race Ready'
                                            : 'Service Required' })] }), _jsxs("div", { style: styles.statusGrid, children: [_jsxs("div", { style: styles.statusItem, children: [_jsx("span", { children: "Tire Sets Available:" }), _jsxs("strong", { children: [car.tire_sets_available, " / 4 minimum"] })] }), _jsxs("div", { style: styles.statusItem, children: [_jsx("span", { children: "Engine Condition:" }), _jsx("strong", { style: { color: car.needs_engine_rebuild ? '#ef4444' : '#22c55e' }, children: car.needs_engine_rebuild ? 'Needs Rebuild' : 'OK' })] }), _jsxs("div", { style: styles.statusItem, children: [_jsx("span", { children: "Gearbox Service:" }), _jsx("strong", { style: { color: car.needs_gearbox_maint ? '#ef4444' : '#22c55e' }, children: car.needs_gearbox_maint ? 'Service Required' : 'OK' })] }), _jsxs("div", { style: styles.statusItem, children: [_jsx("span", { children: "Oil Status:" }), _jsx("strong", { style: { color: car.needs_oil_change ? '#ef4444' : '#22c55e' }, children: car.needs_oil_change ? 'Oil Change Needed' : 'Fresh' })] })] }), _jsxs("div", { style: styles.actionSection, children: [_jsxs("button", { style: styles.actionBtn, disabled: !car.needs_oil_change || player.budget < car.oil_change_cost, onClick: () => handleMaintenance(car.id, 'OilChange'), children: ["Oil Change (\u00A3", car.oil_change_cost, ")"] }), _jsxs("button", { style: styles.actionBtn, disabled: !car.needs_engine_rebuild || player.budget < car.engine_rebuild_cost, onClick: () => handleMaintenance(car.id, 'EngineRebuild'), children: ["Rebuild Engine (\u00A3", car.engine_rebuild_cost, ")"] }), _jsxs("button", { style: styles.actionBtn, disabled: !car.needs_gearbox_maint || player.budget < car.gearbox_maint_cost, onClick: () => handleMaintenance(car.id, 'GearboxService'), children: ["Service Gearbox (\u00A3", car.gearbox_maint_cost, ")"] }), _jsxs("div", { style: styles.tirePurchaseRow, children: [_jsxs("div", { style: styles.tireInputGroup, children: [_jsx("label", { style: { fontSize: '0.8rem', color: '#94a3b8' }, children: "Buy Tires:" }), _jsx("input", { type: "number", min: "1", max: "20", value: selectedTires, onChange: (e) => handleTireCountChange(car.id, parseInt(e.target.value) || 1), style: styles.numberInput }), _jsxs("span", { style: { fontSize: '0.85rem' }, children: ["sets (\u00A3", totalTireCost, ")"] })] }), _jsx("button", { style: styles.actionBtn, disabled: player.budget < totalTireCost, onClick: () => handleMaintenance(car.id, { BuyTires: selectedTires }), children: "Purchase Tires" })] })] })] }, car.id));
                }) })] }));
};
const styles = {
    card: {
        padding: '1rem',
        borderRadius: '8px',
        backgroundColor: '#1e293b',
        color: '#f8fafc',
        marginBottom: '1rem',
    },
    errorBox: {
        padding: '0.75rem',
        backgroundColor: '#7f1d1d',
        color: '#fca5a5',
        borderRadius: '4px',
        marginBottom: '1rem',
        fontSize: '0.9rem',
    },
    carGrid: {
        display: 'flex',
        flexDirection: 'column',
        gap: '1rem',
    },
    carCard: {
        padding: '1rem',
        borderRadius: '6px',
        backgroundColor: '#0f172a',
        border: '1px solid #334155',
    },
    carHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '1rem',
    },
    badgeReady: {
        backgroundColor: '#15803d',
        color: '#ffffff',
        padding: '0.25rem 0.5rem',
        borderRadius: '4px',
        fontSize: '0.75rem',
        fontWeight: 'bold',
    },
    badgeNotReady: {
        backgroundColor: '#b91c1c',
        color: '#ffffff',
        padding: '0.25rem 0.5rem',
        borderRadius: '4px',
        fontSize: '0.75rem',
        fontWeight: 'bold',
    },
    statusGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
        gap: '0.5rem',
        marginBottom: '1rem',
        fontSize: '0.875rem',
        backgroundColor: '#1e293b',
        padding: '0.75rem',
        borderRadius: '4px',
    },
    statusItem: {
        display: 'flex',
        justifyContent: 'space-between',
    },
    actionSection: {
        display: 'flex',
        flexDirection: 'column',
        gap: '0.5rem',
    },
    tirePurchaseRow: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        flexWrap: 'wrap',
        gap: '0.5rem',
        marginTop: '0.5rem',
        paddingTop: '0.5rem',
        borderTop: '1px dashed #334155',
    },
    tireInputGroup: {
        display: 'flex',
        alignItems: 'center',
        gap: '0.5rem',
    },
    numberInput: {
        width: '50px',
        padding: '0.25rem',
        backgroundColor: '#1e293b',
        color: '#ffffff',
        border: '1px solid #475569',
        borderRadius: '4px',
        textAlign: 'center',
    },
    actionBtn: {
        padding: '0.5rem 0.75rem',
        borderRadius: '4px',
        border: '1px solid #3b82f6',
        backgroundColor: '#2563eb',
        color: '#ffffff',
        cursor: 'pointer',
        fontWeight: 'bold',
    },
};
//# sourceMappingURL=Garage.js.map