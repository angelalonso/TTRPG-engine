import React from 'react';

interface ClickableTableCellProps {
  children: React.ReactNode;
  onClick: React.MouseEventHandler<HTMLTableCellElement>;
  style?: React.CSSProperties;
}

export const ClickableTableCell: React.FC<ClickableTableCellProps> = ({
  children,
  onClick,
  style,
}) => (
  <td
    style={{ ...style, cursor: 'pointer' }}
    onClick={onClick}
  >
    {children}
  </td>
);
