import React from 'react';
import type { GameState } from '../types/game';
interface TimeControlProps {
    gameState: GameState;
    onStateUpdate: (newState: GameState) => void;
}
export declare const TimeControl: React.FC<TimeControlProps>;
export {};
//# sourceMappingURL=TimeControl.d.ts.map