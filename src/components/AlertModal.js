import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import React from 'react';
import { dismissAlert } from '../services/tauriApi';
export const AlertModal = ({ alerts, onDismiss }) => {
    if (!alerts || alerts.length === 0)
        return null;
    const currentAlert = alerts[0];
    const handleDismiss = async () => {
        try {
            const newState = await dismissAlert(currentAlert.id);
            onDismiss(newState);
        }
        catch (err) {
            console.error('Failed to dismiss alert:', err);
        }
    };
    return (_jsx("div", { style: styles.overlay, children: _jsxs("div", { style: styles.modal, children: [_jsxs("div", { style: styles.header, children: [_jsx("h3", { style: styles.title, children: currentAlert.title }), alerts.length > 1 && (_jsxs("span", { style: styles.badge, children: [alerts.length, " Pending"] }))] }), _jsx("p", { style: styles.message, children: currentAlert.message }), _jsxs("div", { style: styles.footer, children: [_jsx("span", { style: styles.pauseNote, children: "\u23F8\uFE0F Time automatically paused" }), _jsx("button", { style: styles.button, onClick: handleDismiss, children: "Acknowledge & Continue" })] })] }) }));
};
export default AlertModal;
const styles = {
    overlay: {
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(15, 23, 42, 0.85)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 9999,
        backdropFilter: 'blur(4px)',
    },
    modal: {
        backgroundColor: '#1e293b',
        border: '2px solid #3b82f6',
        borderRadius: '12px',
        padding: '1.5rem',
        maxWidth: '480px',
        width: '90%',
        boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.5)',
    },
    header: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '1rem',
    },
    title: {
        margin: 0,
        color: '#f8fafc',
        fontSize: '1.25rem',
    },
    badge: {
        backgroundColor: '#3b82f6',
        color: '#ffffff',
        fontSize: '0.75rem',
        padding: '0.2rem 0.5rem',
        borderRadius: '12px',
        fontWeight: 'bold',
    },
    message: {
        color: '#cbd5e1',
        fontSize: '1rem',
        lineHeight: '1.5',
        marginBottom: '1.5rem',
    },
    footer: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
    },
    pauseNote: {
        fontSize: '0.8rem',
        color: '#f59e0b',
    },
    button: {
        backgroundColor: '#2563eb',
        color: '#ffffff',
        border: 'none',
        padding: '0.6rem 1.2rem',
        borderRadius: '6px',
        fontWeight: 'bold',
        cursor: 'pointer',
    },
};
//# sourceMappingURL=AlertModal.js.map