import React from 'react';
import type { GameAlert, GameState } from '../types/game';
interface AlertModalProps {
    alerts: GameAlert[];
    onDismiss: (updatedState: GameState) => void;
}
export declare const AlertModal: React.FC<AlertModalProps>;
export default AlertModal;
//# sourceMappingURL=AlertModal.d.ts.map