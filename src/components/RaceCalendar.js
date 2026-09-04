import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import React, { useState } from 'react';
import { enterRace, getGameState } from '../services/tauriApi';
export const RaceCalendar = ({ gameState, onStateUpdate, }) => {
    const { current_day, catalog, player } = gameState;
    const currentDayInYear = ((current_day - 1) % 365) + 1;
    const [selectedCarId, setSelectedCarId] = useState(player.cars[0]?.id ?? '');
    const [raceResult, setRaceResult] = useState(null);
    const [error, setError] = useState(null);
    const [isSubmitting, setIsSubmitting] = useState(false);
    const selectedCar = player.cars.find((c) => c.id === selectedCarId);
    const handleEnterRace = async (raceId) => {
        if (!selectedCarId) {
            setError('Please select a car from your garage first.');
            return;
        }
        try {
            setError(null);
            setIsSubmitting(true);
            const result = await enterRace(selectedCarId, raceId);
            setRaceResult(result);
            // Refresh global state after race execution
            const updatedState = await getGameState();
            onStateUpdate(updatedState);
        }
        catch (err) {
            setError(String(err));
        }
        finally {
            setIsSubmitting(false);
        }
    };
    const formatPosition = (pos) => {
        if (typeof pos === 'string') {
            switch (pos) {
                case 'First':
                    return '🥇 1st Place';
                case 'Second':
                    return '🥈 2nd Place';
                case 'Third':
                    return '🥉 3rd Place';
                default:
                    return 'Unplaced';
            }
        }
        if (pos && typeof pos === 'object' && 'DNF' in pos) {
            return `❌ DNF (${pos.DNF})`;
        }
        return 'Unknown';
    };
    return (_jsxs("div", { style: styles.card, children: [_jsx("h2", { children: "\uD83C\uDFC1 Race Calendar & Events" }), _jsxs("p", { style: { color: '#94a3b8', fontSize: '0.9rem' }, children: ["Today is ", _jsxs("strong", { children: ["Day ", currentDayInYear] }), " of the year. Races only take place on their exact scheduled day."] }), error && _jsx("div", { style: styles.errorBox, children: error }), raceResult && (_jsxs("div", { style: styles.resultBox, children: [_jsxs("div", { style: styles.resultHeader, children: [_jsxs("h3", { style: { margin: 0 }, children: [raceResult.race_name, " - Result"] }), _jsx("button", { onClick: () => setRaceResult(null), style: styles.closeBtn, children: "\u2715" })] }), _jsx("div", { style: { fontSize: '1.2rem', fontWeight: 'bold', margin: '0.5rem 0' }, children: formatPosition(raceResult.position) }), _jsx("p", { style: { margin: '0.25rem 0' }, children: raceResult.message }), _jsxs("div", { style: styles.resultFinancials, children: [_jsxs("span", { children: ["Entry Fee: -\u00A3", raceResult.entry_fee_paid] }), _jsxs("span", { style: { color: raceResult.prize_awarded > 0 ? '#22c55e' : '#94a3b8' }, children: ["Prize: +\u00A3", raceResult.prize_awarded] })] })] })), _jsxs("div", { style: styles.carSelectSection, children: [_jsx("label", { style: { fontWeight: 'bold', fontSize: '0.9rem' }, children: "Select Race Vehicle:" }), player.cars.length === 0 ? (_jsx("span", { style: { color: '#ef4444', fontSize: '0.875rem' }, children: "No cars in garage! Buy one from the Dealership." })) : (_jsx("select", { value: selectedCarId, onChange: (e) => setSelectedCarId(e.target.value), style: styles.selectInput, children: player.cars.map((car) => (_jsxs("option", { value: car.id, children: [car.name, " (", car.tire_sets_available, " tires,", ' ', car.needs_oil_change || car.needs_engine_rebuild || car.needs_gearbox_maint
                                    ? 'Needs Maint'
                                    : 'Ready', ")"] }, car.id))) }))] }), _jsx("div", { style: styles.raceList, children: catalog.races.map((race) => {
                    const daysUntil = ((race.day_of_year - currentDayInYear + 365) % 365);
                    const isToday = daysUntil === 0;
                    const isCarReady = selectedCar &&
                        !selectedCar.needs_oil_change &&
                        !selectedCar.needs_engine_rebuild &&
                        !selectedCar.needs_gearbox_maint &&
                        selectedCar.tire_sets_available >= 4;
                    const hasFunds = player.budget >= race.entry_fee;
                    const canEnter = isToday && isCarReady && hasFunds && !isSubmitting;
                    return (_jsxs("div", { style: {
                            ...styles.raceCard,
                            borderColor: isToday ? '#3b82f6' : '#334155',
                            backgroundColor: isToday ? '#1e293b' : '#0f172a',
                        }, children: [_jsxs("div", { style: styles.raceInfo, children: [_jsxs("div", { style: { display: 'flex', alignItems: 'center', gap: '0.5rem' }, children: [_jsx("h3", { style: { margin: 0 }, children: race.name }), isToday && _jsx("span", { style: styles.todayBadge, children: "TODAY" })] }), _jsxs("div", { style: styles.raceMeta, children: [_jsxs("span", { children: ["Scheduled: Day ", race.day_of_year] }), _jsxs("span", { children: ["Entry Fee: \u00A3", race.entry_fee] }), _jsxs("span", { children: ["Prize Pool: \u00A3", race.prize_pool] })] })] }), _jsxs("div", { style: styles.actionCol, children: [!isToday && (_jsxs("span", { style: styles.countdownText, children: ["In ", daysUntil, " ", daysUntil === 1 ? 'day' : 'days'] })), _jsx("button", { onClick: () => handleEnterRace(race.id), disabled: !canEnter, style: canEnter ? styles.enterBtn : styles.disabledBtn, children: isSubmitting ? 'Simulating...' : 'Enter Race' }), isToday && !canEnter && (_jsx("span", { style: styles.reasonText, children: !hasFunds
                                            ? 'Low Budget'
                                            : !isCarReady
                                                ? 'Car Not Ready'
                                                : 'Cannot Enter' }))] })] }, race.id));
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
    resultBox: {
        padding: '1rem',
        backgroundColor: '#0f172a',
        border: '2px solid #3b82f6',
        borderRadius: '6px',
        marginBottom: '1rem',
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
    carSelectSection: {
        display: 'flex',
        alignItems: 'center',
        gap: '1rem',
        marginBottom: '1rem',
        backgroundColor: '#0f172a',
        padding: '0.75rem',
        borderRadius: '6px',
    },
    selectInput: {
        padding: '0.5rem',
        borderRadius: '4px',
        backgroundColor: '#1e293b',
        color: '#ffffff',
        border: '1px solid #475569',
        flex: 1,
    },
    raceList: {
        display: 'flex',
        flexDirection: 'column',
        gap: '0.75rem',
    },
    raceCard: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '1rem',
        borderRadius: '6px',
        border: '1px solid #334155',
    },
    raceInfo: {
        display: 'flex',
        flexDirection: 'column',
        gap: '0.25rem',
    },
    raceMeta: {
        display: 'flex',
        gap: '1rem',
        fontSize: '0.85rem',
        color: '#94a3b8',
    },
    todayBadge: {
        backgroundColor: '#2563eb',
        color: '#ffffff',
        fontSize: '0.7rem',
        fontWeight: 'bold',
        padding: '0.1rem 0.4rem',
        borderRadius: '3px',
    },
    actionCol: {
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'flex-end',
        gap: '0.25rem',
    },
    countdownText: {
        fontSize: '0.8rem',
        color: '#64748b',
    },
    reasonText: {
        fontSize: '0.75rem',
        color: '#ef4444',
    },
    enterBtn: {
        padding: '0.5rem 1rem',
        borderRadius: '4px',
        border: 'none',
        backgroundColor: '#16a34a',
        color: '#ffffff',
        fontWeight: 'bold',
        cursor: 'pointer',
    },
    disabledBtn: {
        padding: '0.5rem 1rem',
        borderRadius: '4px',
        border: '1px solid #475569',
        backgroundColor: '#334155',
        color: '#64748b',
        cursor: 'not-allowed',
    },
};
//# sourceMappingURL=RaceCalendar.js.map