import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import React, { useEffect } from 'react';
import { setTimeSpeed, tickGameDay } from '../services/tauriApi';
const SPEED_INTERVALS = {
    Paused: null,
    OneDayEveryFiveSec: 5000,
    OneDayPerSec: 1000,
    OneWeekPerSec: 142, // ~7 ticks per second (1 week / sec)
    RealTime: 1000,
};
export const TimeControl = ({ gameState, onStateUpdate, }) => {
    const { current_day, time_speed, player } = gameState;
    // Calculate calendar & age metrics
    const year = Math.floor((current_day - 1) / 365) + 1;
    const dayOfYear = ((current_day - 1) % 365) + 1;
    const ageYears = Math.floor(player.age_days / 365);
    const ageDaysRemaining = player.age_days % 365;
    // Time-tick engine effect
    useEffect(() => {
        const intervalMs = SPEED_INTERVALS[time_speed];
        if (intervalMs === null)
            return;
        const timer = setInterval(async () => {
            try {
                const updatedState = await tickGameDay();
                onStateUpdate(updatedState);
            }
            catch (err) {
                console.error('Tick execution failed:', err);
            }
        }, intervalMs);
        return () => clearInterval(timer);
    }, [time_speed, onStateUpdate]);
    const handleSpeedChange = async (speed) => {
        try {
            await setTimeSpeed(speed);
            onStateUpdate({ ...gameState, time_speed: speed });
        }
        catch (err) {
            console.error('Failed to change speed:', err);
        }
    };
    return (_jsxs("div", { style: styles.card, children: [_jsxs("div", { style: styles.statsGrid, children: [_jsxs("div", { style: styles.statBox, children: [_jsx("span", { style: styles.label, children: "YEAR" }), _jsx("div", { style: styles.value, children: year })] }), _jsxs("div", { style: styles.statBox, children: [_jsx("span", { style: styles.label, children: "DAY" }), _jsxs("div", { style: styles.value, children: [dayOfYear, " ", _jsx("span", { style: styles.subtext, children: "/ 365" })] })] }), _jsxs("div", { style: styles.statBox, children: [_jsx("span", { style: styles.label, children: "CHARACTER AGE" }), _jsxs("div", { style: styles.value, children: [ageYears, "y ", ageDaysRemaining, "d"] })] }), _jsxs("div", { style: styles.statBox, children: [_jsx("span", { style: styles.label, children: "BALANCE" }), _jsxs("div", { style: styles.value, children: ["\u00A3", player.budget.toLocaleString()] })] })] }), _jsxs("div", { style: styles.controlsRow, children: [_jsx("button", { style: time_speed === 'Paused' ? styles.activeBtn : styles.btn, onClick: () => handleSpeedChange('Paused'), children: "\u23F8 Pause" }), _jsx("button", { style: time_speed === 'OneDayEveryFiveSec' ? styles.activeBtn : styles.btn, onClick: () => handleSpeedChange('OneDayEveryFiveSec'), children: "\u25B6 1d / 5s" }), _jsx("button", { style: time_speed === 'OneDayPerSec' ? styles.activeBtn : styles.btn, onClick: () => handleSpeedChange('OneDayPerSec'), children: "\u25B6\u25B6 1d / 1s" }), _jsx("button", { style: time_speed === 'OneWeekPerSec' ? styles.activeBtn : styles.btn, onClick: () => handleSpeedChange('OneWeekPerSec'), children: "\u23E9 1w / 1s" }), time_speed === 'RealTime' && (_jsx("span", { style: styles.eventBadge, children: "\u26A0\uFE0F Real-Time Mode (Automatic Event Triggered)" }))] })] }));
};
const styles = {
    card: {
        padding: '1rem',
        borderRadius: '8px',
        backgroundColor: '#1e293b',
        color: '#f8fafc',
        marginBottom: '1rem',
    },
    statsGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(120px, 1fr))',
        gap: '1rem',
        marginBottom: '1rem',
    },
    statBox: {
        display: 'flex',
        flexDirection: 'column',
    },
    label: {
        fontSize: '0.75rem',
        color: '#94a3b8',
        fontWeight: 'bold',
    },
    value: {
        fontSize: '1.25rem',
        fontWeight: 'bold',
    },
    subtext: {
        fontSize: '0.85rem',
        color: '#64748b',
    },
    controlsRow: {
        display: 'flex',
        gap: '0.5rem',
        alignItems: 'center',
        flexWrap: 'wrap',
    },
    btn: {
        padding: '0.5rem 0.75rem',
        borderRadius: '4px',
        border: '1px solid #475569',
        backgroundColor: '#334155',
        color: '#ffffff',
        cursor: 'pointer',
    },
    activeBtn: {
        padding: '0.5rem 0.75rem',
        borderRadius: '4px',
        border: '1px solid #3b82f6',
        backgroundColor: '#2563eb',
        color: '#ffffff',
        fontWeight: 'bold',
        cursor: 'pointer',
    },
    eventBadge: {
        padding: '0.5rem 0.75rem',
        borderRadius: '4px',
        backgroundColor: '#ef4444',
        color: '#ffffff',
        fontSize: '0.85rem',
        fontWeight: 'bold',
    },
};
//# sourceMappingURL=TimeControl.js.map