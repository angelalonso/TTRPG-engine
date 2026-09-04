import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";
import React from 'react';
export const CharacterProfile = ({ gameState }) => {
    const { player, catalog, current_day } = gameState;
    const ageYears = Math.floor(player.age_days / 365);
    const remainingDays = player.age_days % 365;
    const getIntervalDays = (freq, unit) => {
        const u = unit.trim().toLowerCase();
        if (u.startsWith('day'))
            return freq * 1;
        if (u.startsWith('month'))
            return freq * 30;
        if (u.startsWith('year'))
            return freq * 365;
        return freq;
    };
    const getNextPayoutInfo = (active, action) => {
        const interval = getIntervalDays(action.payout_freq, action.payout_freq_unit);
        if (interval <= 0)
            return { nextDay: current_day, daysRemaining: 0 };
        const daysElapsed = current_day - active.start_day;
        const remainder = daysElapsed % interval;
        const daysRemaining = remainder === 0 && daysElapsed > 0 ? 0 : interval - remainder;
        const nextDay = current_day + daysRemaining;
        return { nextDay, daysRemaining };
    };
    return (_jsxs("div", { style: styles.container, children: [_jsxs("div", { style: styles.card, children: [_jsx("h2", { style: styles.cardTitle, children: "\uD83D\uDC64 Character Profile" }), _jsxs("div", { style: styles.statsGrid, children: [_jsxs("div", { children: [_jsx("span", { style: styles.label, children: "Age:" }), _jsxs("div", { style: styles.statValue, children: [ageYears, " yrs, ", remainingDays, " days"] })] }), _jsxs("div", { children: [_jsx("span", { style: styles.label, children: "Current Budget:" }), _jsxs("div", { style: { ...styles.statValue, color: '#22c55e' }, children: ["\u00A3", player.budget.toLocaleString()] })] }), _jsxs("div", { children: [_jsx("span", { style: styles.label, children: "Current Day:" }), _jsxs("div", { style: styles.statValue, children: ["Day ", current_day] })] }), _jsxs("div", { children: [_jsx("span", { style: styles.label, children: "Garage Cars:" }), _jsxs("div", { style: styles.statValue, children: [player.cars.length, " Vehicles"] })] })] })] }), _jsxs("div", { style: styles.card, children: [_jsxs("div", { style: styles.sectionHeader, children: [_jsx("h3", { style: { margin: 0 }, children: "\uD83D\uDCBC Ongoing Jobs & Active Situations" }), _jsxs("span", { style: styles.countBadge, children: [player.active_actions?.length || 0, " Active"] })] }), (!player.active_actions || player.active_actions.length === 0) ? (_jsx("div", { style: styles.emptyState, children: "No ongoing jobs or active situations. Visit the Actions panel to start a job or sponsor!" })) : (_jsx("div", { style: styles.actionsList, children: player.active_actions.map((active) => {
                            const actionData = catalog.actions.find((a) => a.id === active.action_id);
                            if (!actionData)
                                return null;
                            const { nextDay, daysRemaining } = getNextPayoutInfo(active, actionData);
                            return (_jsx("div", { style: styles.actionItem, children: _jsxs("div", { style: styles.actionMain, children: [_jsxs("div", { children: [_jsxs("div", { style: styles.actionName, children: [actionData.name, ' ', _jsx("span", { style: styles.typeBadge, children: actionData.type })] }), _jsxs("div", { style: styles.actionMeta, children: ["Started on Day ", active.start_day, " \u2022 Payout: \u00A3", actionData.payout.toLocaleString(), " every ", actionData.payout_freq, ' ', actionData.payout_freq_unit, "(s)"] })] }), _jsxs("div", { style: styles.payoutBadgeBox, children: [_jsx("span", { style: styles.label, children: "Next Payout:" }), _jsx("div", { style: styles.payoutDayText, children: daysRemaining === 0 ? (_jsx("span", { style: { color: '#22c55e', fontWeight: 'bold' }, children: "Payout Due Today!" })) : (_jsxs(_Fragment, { children: ["Day ", nextDay, ' ', _jsxs("span", { style: styles.countdownText, children: ["(", daysRemaining, " days left)"] })] })) })] })] }) }, active.action_id));
                        }) }))] })] }));
};
const styles = {
    container: {
        display: 'flex',
        flexDirection: 'column',
        gap: '1.25rem',
    },
    card: {
        backgroundColor: '#1e293b',
        border: '1px solid #334155',
        borderRadius: '8px',
        padding: '1.25rem',
    },
    cardTitle: {
        margin: '0 0 1rem 0',
        color: '#f8fafc',
        fontSize: '1.25rem',
    },
    statsGrid: {
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))',
        gap: '1rem',
    },
    label: {
        fontSize: '0.75rem',
        color: '#94a3b8',
        display: 'block',
        marginBottom: '0.2rem',
    },
    statValue: {
        fontSize: '1.1rem',
        fontWeight: 'bold',
        color: '#f1f5f9',
    },
    sectionHeader: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '1rem',
        color: '#f8fafc',
    },
    countBadge: {
        backgroundColor: '#3b82f6',
        color: '#ffffff',
        padding: '0.2rem 0.6rem',
        borderRadius: '12px',
        fontSize: '0.8rem',
        fontWeight: 'bold',
    },
    emptyState: {
        padding: '1.5rem',
        backgroundColor: '#0f172a',
        borderRadius: '6px',
        color: '#64748b',
        textAlign: 'center',
        fontSize: '0.9rem',
        border: '1px dashed #334155',
    },
    actionsList: {
        display: 'flex',
        flexDirection: 'column',
        gap: '0.75rem',
    },
    actionItem: {
        backgroundColor: '#0f172a',
        border: '1px solid #334155',
        borderRadius: '6px',
        padding: '1rem',
    },
    actionMain: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        flexWrap: 'wrap',
        gap: '0.75rem',
    },
    actionName: {
        fontSize: '1rem',
        fontWeight: 'bold',
        color: '#38bdf8',
        display: 'flex',
        alignItems: 'center',
        gap: '0.5rem',
    },
    typeBadge: {
        fontSize: '0.7rem',
        backgroundColor: '#334155',
        color: '#cbd5e1',
        padding: '0.15rem 0.4rem',
        borderRadius: '4px',
        fontWeight: 'normal',
        textTransform: 'uppercase',
    },
    actionMeta: {
        fontSize: '0.825rem',
        color: '#94a3b8',
        marginTop: '0.25rem',
    },
    payoutBadgeBox: {
        backgroundColor: '#1e293b',
        padding: '0.5rem 0.75rem',
        borderRadius: '6px',
        border: '1px solid #334155',
        textAlign: 'right',
    },
    payoutDayText: {
        fontSize: '0.9rem',
        color: '#f8fafc',
    },
    countdownText: {
        color: '#f59e0b',
        fontSize: '0.8rem',
    },
};
//# sourceMappingURL=CharacterProfile.js.map