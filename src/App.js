import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import React, { useEffect, useState } from 'react';
import { getGameState, setTimeSpeed, tickGameDay, buyCar, maintainCar, performAction, enterRace, } from './services/tauriApi';
import { AlertModal } from './components/AlertModal';
export function App() {
    const [gameState, setGameState] = useState(null);
    const [activeTab, setActiveTab] = useState('dashboard');
    const [selectedCarId, setSelectedCarId] = useState('');
    const [feedbackMessage, setFeedbackMessage] = useState('');
    // Initial Game State Load
    useEffect(() => {
        getGameState()
            .then(setGameState)
            .catch((err) => console.error('Failed to load initial state:', err));
    }, []);
    // Time Engine Loop
    useEffect(() => {
        if (!gameState || gameState.time_speed === 'Paused')
            return;
        let intervalMs = 1000;
        if (gameState.time_speed === 'OneDayEveryFiveSec')
            intervalMs = 5000;
        if (gameState.time_speed === 'OneDayPerSec')
            intervalMs = 1000;
        if (gameState.time_speed === 'OneWeekPerSec')
            intervalMs = 1000 / 7;
        const timer = setInterval(() => {
            tickGameDay()
                .then(setGameState)
                .catch((err) => console.error('Tick failed:', err));
        }, intervalMs);
        return () => clearInterval(timer);
    }, [gameState?.time_speed]);
    if (!gameState) {
        return (_jsx("div", { style: styles.loading, children: _jsx("h2", { children: "\uD83C\uDFCE\uFE0F Loading Engine State..." }) }));
    }
    const currentDayOfYear = ((gameState.current_day - 1) % 365) + 1;
    const currentYear = Math.floor((gameState.current_day - 1) / 365) + 1;
    const handleSpeedChange = async (speed) => {
        try {
            const updated = await setTimeSpeed(speed);
            setGameState(updated);
        }
        catch (err) {
            setFeedbackMessage(`Failed to change speed: ${err}`);
        }
    };
    const handleBuyCar = async (carId) => {
        try {
            const updated = await buyCar(carId);
            setGameState(updated);
            setFeedbackMessage('Vehicle purchased successfully!');
        }
        catch (err) {
            setFeedbackMessage(`Purchase failed: ${err}`);
        }
    };
    const handleMaintainCar = async (carId, maintType) => {
        try {
            const updated = await maintainCar(carId, maintType);
            setGameState(updated);
            setFeedbackMessage('Maintenance completed successfully!');
        }
        catch (err) {
            setFeedbackMessage(`Maintenance failed: ${err}`);
        }
    };
    const handlePerformAction = async (actionId) => {
        try {
            const res = await performAction(actionId);
            setFeedbackMessage(res.message);
            const freshState = await getGameState();
            setGameState(freshState);
        }
        catch (err) {
            setFeedbackMessage(`Action failed: ${err}`);
        }
    };
    const handleEnterRace = async (raceId) => {
        if (!selectedCarId) {
            setFeedbackMessage('Please select a car from your garage before entering a race.');
            return;
        }
        try {
            const res = await enterRace(selectedCarId, raceId);
            setFeedbackMessage(`${res.message} (Prize: £${res.prize_awarded})`);
            const freshState = await getGameState();
            setGameState(freshState);
        }
        catch (err) {
            setFeedbackMessage(`Race entry failed: ${err}`);
        }
    };
    return (_jsxs("div", { style: styles.container, children: [_jsxs("header", { style: styles.header, children: [_jsxs("div", { children: [_jsx("h1", { style: styles.appTitle, children: "GTR2 RACEWARS" }), _jsxs("p", { style: styles.subTitle, children: ["Year ", currentYear, ", Day ", currentDayOfYear, " (Total Day ", gameState.current_day, ")"] })] }), _jsxs("div", { style: styles.budgetBadge, children: ["\uD83D\uDCB0 Budget: \u00A3", gameState.player.budget.toLocaleString('en-US', { minimumFractionDigits: 2 })] })] }), _jsxs("div", { style: styles.controlsBar, children: [_jsx("span", { style: { fontWeight: 'bold', color: '#94a3b8' }, children: "Simulation Speed:" }), ['Paused', 'OneDayEveryFiveSec', 'OneDayPerSec', 'OneWeekPerSec'].map((speed) => (_jsx("button", { style: {
                            ...styles.speedButton,
                            backgroundColor: gameState.time_speed === speed ? '#2563eb' : '#334155',
                        }, onClick: () => handleSpeedChange(speed), children: speed === 'Paused' ? '⏸️ Pause' : speed }, speed)))] }), feedbackMessage && (_jsxs("div", { style: styles.feedbackBanner, onClick: () => setFeedbackMessage(''), children: [_jsx("span", { children: feedbackMessage }), _jsx("span", { style: { cursor: 'pointer', fontWeight: 'bold' }, children: "\u2715" })] })), _jsxs("nav", { style: styles.navTabs, children: [_jsx("button", { style: { ...styles.tabButton, borderBottom: activeTab === 'dashboard' ? '3px solid #3b82f6' : 'none' }, onClick: () => setActiveTab('dashboard'), children: "\uD83D\uDCCA Overview" }), _jsx("button", { style: { ...styles.tabButton, borderBottom: activeTab === 'garage' ? '3px solid #3b82f6' : 'none' }, onClick: () => setActiveTab('garage'), children: "\uD83C\uDFCE\uFE0F Garage & Dealership" }), _jsx("button", { style: { ...styles.tabButton, borderBottom: activeTab === 'races' ? '3px solid #3b82f6' : 'none' }, onClick: () => setActiveTab('races'), children: "\uD83C\uDFC1 Race Events" }), _jsx("button", { style: { ...styles.tabButton, borderBottom: activeTab === 'activities' ? '3px solid #3b82f6' : 'none' }, onClick: () => setActiveTab('activities'), children: "\uD83D\uDCBC Jobs & Activities" })] }), _jsxs("main", { style: styles.mainContent, children: [activeTab === 'dashboard' && (_jsxs("div", { style: styles.gridTwoColumn, children: [_jsxs("div", { style: styles.card, children: [_jsx("h3", { children: "Active Employment & Contracts" }), gameState.player.active_actions.length === 0 ? (_jsx("p", { style: { color: '#94a3b8' }, children: "No active recurring jobs. Visit Jobs & Activities to start one." })) : (gameState.player.active_actions.map((act) => {
                                        const spec = gameState.catalog.actions.find((a) => a.id === act.action_id);
                                        const daysActive = gameState.current_day - act.start_day;
                                        return (_jsxs("div", { style: styles.itemRow, children: [_jsxs("div", { children: [_jsx("strong", { children: spec?.name || act.action_id }), _jsxs("div", { style: { fontSize: '0.85rem', color: '#94a3b8' }, children: ["Started Day ", act.start_day, " (", daysActive, " days active)"] })] }), _jsxs("div", { style: { color: '#10b981', fontWeight: 'bold' }, children: ["+\u00A3", spec?.payout, " / ", spec?.payout_freq, " ", spec?.payout_freq_unit] })] }, act.action_id));
                                    }))] }), _jsxs("div", { style: styles.card, children: [_jsx("h3", { children: "Upcoming Race Schedule" }), gameState.catalog.races.map((race) => {
                                        let daysLeft = race.day_of_year - currentDayOfYear;
                                        if (daysLeft < 0)
                                            daysLeft += 365;
                                        return (_jsxs("div", { style: styles.itemRow, children: [_jsxs("div", { children: [_jsx("strong", { children: race.name }), _jsxs("div", { style: { fontSize: '0.85rem', color: '#94a3b8' }, children: ["Day ", race.day_of_year, " of Year (", daysLeft === 0 ? 'TODAY!' : `${daysLeft} days away`, ")"] })] }), _jsxs("div", { style: { textAlign: 'right' }, children: [_jsxs("div", { style: { color: '#3b82f6' }, children: ["Prize: \u00A3", race.prize_pool] }), _jsxs("div", { style: { fontSize: '0.8rem', color: '#94a3b8' }, children: ["Fee: \u00A3", race.entry_fee] })] })] }, race.id));
                                    })] })] })), activeTab === 'garage' && (_jsxs("div", { children: [_jsx("h3", { children: "Your Garage" }), gameState.player.cars.length === 0 ? (_jsx("p", { style: { color: '#94a3b8' }, children: "Your garage is empty. Purchase a car below." })) : (_jsx("div", { style: styles.gridTwoColumn, children: gameState.player.cars.map((car) => (_jsxs("div", { style: {
                                        ...styles.card,
                                        border: selectedCarId === car.id ? '2px solid #3b82f6' : '1px solid #334155',
                                    }, onClick: () => setSelectedCarId(car.id), children: [_jsxs("div", { style: { display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem' }, children: [_jsx("h4", { children: car.name }), selectedCarId === car.id && _jsx("span", { style: { color: '#3b82f6' }, children: "Selected for Race" })] }), _jsxs("p", { style: { margin: '0.2rem 0', fontSize: '0.9rem' }, children: ["\uD83D\uDEDE Tires Available: ", car.tire_sets_available, " sets"] }), _jsxs("p", { style: { margin: '0.2rem 0', fontSize: '0.9rem', color: car.needs_oil_change ? '#ef4444' : '#10b981' }, children: ["\uD83D\uDEE2\uFE0F Oil Status: ", car.needs_oil_change ? 'Needs Service' : 'Good'] }), _jsxs("p", { style: { margin: '0.2rem 0', fontSize: '0.9rem', color: car.needs_gearbox_maint ? '#ef4444' : '#10b981' }, children: ["\u2699\uFE0F Gearbox Status: ", car.needs_gearbox_maint ? 'Needs Service' : 'Good'] }), _jsxs("div", { style: { display: 'flex', gap: '0.5rem', marginTop: '1rem', flexWrap: 'wrap' }, children: [car.needs_oil_change && (_jsxs("button", { style: styles.actionBtn, onClick: () => handleMaintainCar(car.id, 'OilChange'), children: ["Oil Change (\u00A3", car.oil_change_cost, ")"] })), car.needs_gearbox_maint && (_jsxs("button", { style: styles.actionBtn, onClick: () => handleMaintainCar(car.id, 'GearboxService'), children: ["Gearbox Service (\u00A3", car.gearbox_maint_cost, ")"] })), _jsxs("button", { style: styles.actionBtn, onClick: () => handleMaintainCar(car.id, { BuyTires: 4 }), children: ["Buy 4 Tires (\u00A3", car.tire_set_cost * 4, ")"] })] })] }, car.id))) })), _jsx("h3", { style: { marginTop: '2rem' }, children: "Car Dealership" }), _jsx("div", { style: styles.gridTwoColumn, children: gameState.catalog.cars.map((car) => (_jsxs("div", { style: styles.card, children: [_jsx("h4", { children: car.name }), _jsxs("p", { style: { color: '#10b981', fontWeight: 'bold' }, children: ["Price: \u00A3", car.price.toLocaleString()] }), _jsx("button", { style: styles.primaryBtn, onClick: () => handleBuyCar(car.id), children: "Buy Vehicle" })] }, car.id))) })] })), activeTab === 'races' && (_jsxs("div", { children: [_jsx("h3", { children: "Scheduled Race Events" }), _jsxs("p", { style: { color: '#94a3b8' }, children: ["Selected Vehicle for Race: ", selectedCarId ? gameState.player.cars.find((c) => c.id === selectedCarId)?.name : 'None selected (select one in Garage)'] }), _jsx("div", { style: styles.gridTwoColumn, children: gameState.catalog.races.map((race) => {
                                    const isRaceDay = race.day_of_year === currentDayOfYear;
                                    return (_jsxs("div", { style: styles.card, children: [_jsx("h4", { children: race.name }), _jsxs("p", { children: ["Day of Year: ", race.day_of_year] }), _jsxs("p", { children: ["Entry Fee: \u00A3", race.entry_fee] }), _jsxs("p", { style: { color: '#10b981' }, children: ["Prize Pool: \u00A3", race.prize_pool] }), _jsx("button", { style: {
                                                    ...styles.primaryBtn,
                                                    backgroundColor: isRaceDay ? '#22c55e' : '#475569',
                                                    cursor: isRaceDay ? 'pointer' : 'not-allowed',
                                                }, disabled: !isRaceDay, onClick: () => handleEnterRace(race.id), children: isRaceDay ? '🏁 Enter Race Today!' : 'Race Closed' })] }, race.id));
                                }) })] })), activeTab === 'activities' && (_jsxs("div", { children: [_jsx("h3", { children: "Available Jobs & Activities" }), _jsx("div", { style: styles.gridTwoColumn, children: gameState.catalog.actions.map((act) => (_jsxs("div", { style: styles.card, children: [_jsx("h4", { children: act.name }), _jsx("p", { style: { color: '#cbd5e1', fontSize: '0.9rem' }, children: act.description }), _jsxs("p", { style: { color: '#10b981' }, children: ["Payout: \u00A3", act.payout, " (", act.payout_freq_type, " every ", act.payout_freq, " ", act.payout_freq_unit, ")"] }), _jsxs("p", { style: { color: '#94a3b8', fontSize: '0.85rem' }, children: ["Upfront Cost: \u00A3", act.base_cost] }), _jsx("button", { style: styles.primaryBtn, onClick: () => handlePerformAction(act.id), children: "Start Activity" })] }, act.id))) })] }))] }), _jsx(AlertModal, { alerts: gameState.pending_alerts || [], onDismiss: setGameState })] }));
}
export default App;
const styles = {
    container: {
        backgroundColor: '#0f172a',
        color: '#f8fafc',
        minHeight: '100vh',
        padding: '1.5rem',
        fontFamily: 'Segoe UI, Tahoma, Geneva, Verdana, sans-serif',
    },
    loading: {
        backgroundColor: '#0f172a',
        color: '#f8fafc',
        height: '100vh',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontSize: '1.25rem',
    },
    header: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        borderBottom: '1px solid #334155',
        paddingBottom: '1rem',
    },
    appTitle: {
        margin: 0,
        fontSize: '1.75rem',
        letterSpacing: '1px',
        color: '#3b82f6',
    },
    subTitle: {
        margin: '0.25rem 0 0 0',
        color: '#94a3b8',
        fontSize: '0.9rem',
    },
    budgetBadge: {
        backgroundColor: '#1e293b',
        border: '1px solid #10b981',
        padding: '0.6rem 1.2rem',
        borderRadius: '8px',
        fontWeight: 'bold',
        fontSize: '1.1rem',
        color: '#10b981',
    },
    controlsBar: {
        display: 'flex',
        alignItems: 'center',
        gap: '0.5rem',
        margin: '1rem 0',
        backgroundColor: '#1e293b',
        padding: '0.75rem',
        borderRadius: '8px',
    },
    speedButton: {
        color: '#ffffff',
        border: 'none',
        padding: '0.4rem 0.8rem',
        borderRadius: '4px',
        cursor: 'pointer',
        fontWeight: 'bold',
    },
    feedbackBanner: {
        backgroundColor: '#3b82f6',
        color: '#ffffff',
        padding: '0.75rem 1rem',
        borderRadius: '6px',
        marginBottom: '1rem',
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
    },
    navTabs: {
        display: 'flex',
        gap: '1rem',
        borderBottom: '1px solid #334155',
        marginBottom: '1.5rem',
    },
    tabButton: {
        backgroundColor: 'transparent',
        color: '#f8fafc',
        border: 'none',
        padding: '0.75rem 1rem',
        fontSize: '1rem',
        cursor: 'pointer',
        fontWeight: 'bold',
    },
    mainContent: {
        marginTop: '1rem',
    },
    gridTwoColumn: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(320px, 1fr))',
        gap: '1rem',
    },
    card: {
        backgroundColor: '#1e293b',
        borderRadius: '8px',
        padding: '1.25rem',
        border: '1px solid #334155',
    },
    itemRow: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '0.75rem 0',
        borderBottom: '1px solid #334155',
    },
    primaryBtn: {
        backgroundColor: '#2563eb',
        color: '#ffffff',
        border: 'none',
        padding: '0.5rem 1rem',
        borderRadius: '4px',
        cursor: 'pointer',
        fontWeight: 'bold',
        marginTop: '0.5rem',
        width: '100%',
    },
    actionBtn: {
        backgroundColor: '#334155',
        color: '#ffffff',
        border: 'none',
        padding: '0.4rem 0.8rem',
        borderRadius: '4px',
        cursor: 'pointer',
        fontSize: '0.85rem',
    },
};
//# sourceMappingURL=App.js.map