import { useState, useEffect, useRef, useCallback } from 'react';
import {
  Search,
  Table2,
  FileJson,
  Share2,
  GitBranch,
  Key,
  Settings,
  LayoutDashboard,
  Home,
  Play,
  Camera,
  Plus,
} from 'lucide-react';

interface Command {
  id: string;
  label: string;
  icon: React.ReactNode;
  action: () => void;
  shortcut?: string;
  group?: string;
}

interface Props {
  open: boolean;
  onClose: () => void;
  onNavigate: (page: string) => void;
  projectOpen: boolean;
}

export default function CommandPalette({ open, onClose, onNavigate, projectOpen }: Props) {
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const commands: Command[] = [
    { id: 'home', label: 'Go to Home', icon: <Home size={16} />, action: () => onNavigate('home'), shortcut: 'Ctrl+1', group: 'Navigate' },
    { id: 'dashboard', label: 'Go to Dashboard', icon: <LayoutDashboard size={16} />, action: () => onNavigate('dashboard'), shortcut: 'Ctrl+2', group: 'Navigate' },
    { id: 'tables', label: 'Open SQL Editor', icon: <Table2 size={16} />, action: () => onNavigate('tables'), shortcut: 'Ctrl+3', group: 'Navigate' },
    { id: 'nosql', label: 'Open NoSQL Browser', icon: <FileJson size={16} />, action: () => onNavigate('nosql'), shortcut: 'Ctrl+4', group: 'Navigate' },
    { id: 'schema', label: 'View Schema Map', icon: <Share2 size={16} />, action: () => onNavigate('schema'), shortcut: 'Ctrl+5', group: 'Navigate' },
    { id: 'migrations', label: 'Open Migrations', icon: <GitBranch size={16} />, action: () => onNavigate('migrations'), shortcut: 'Ctrl+6', group: 'Navigate' },
    { id: 'keys', label: 'Manage API Keys', icon: <Key size={16} />, action: () => onNavigate('keys'), shortcut: 'Ctrl+7', group: 'Navigate' },
    { id: 'settings', label: 'Open Settings', icon: <Settings size={16} />, action: () => onNavigate('settings'), shortcut: 'Ctrl+,', group: 'Navigate' },
    { id: 'run-migrations', label: 'Run Pending Migrations', icon: <Play size={16} />, action: () => onNavigate('migrations'), group: 'Actions' },
    { id: 'snapshot', label: 'Generate Schema Snapshot', icon: <Camera size={16} />, action: () => onNavigate('migrations'), group: 'Actions' },
    { id: 'new-table', label: 'Create New Table', icon: <Plus size={16} />, action: () => onNavigate('tables'), shortcut: 'Ctrl+N', group: 'Actions' },
  ];

  const filteredCommands = query.trim()
    ? commands.filter(c =>
        c.label.toLowerCase().includes(query.toLowerCase()) ||
        c.id.toLowerCase().includes(query.toLowerCase())
      )
    : commands;

  // Only show project-related commands if project is open, always show home/settings
  const displayCommands = projectOpen
    ? filteredCommands
    : filteredCommands.filter(c => ['home', 'settings'].includes(c.id));

  const handleSelect = useCallback((cmd: Command) => {
    cmd.action();
    onClose();
    setQuery('');
    setSelectedIndex(0);
  }, [onClose]);

  useEffect(() => {
    if (open && inputRef.current) {
      inputRef.current.focus();
      setQuery('');
      setSelectedIndex(0);
    }
  }, [open]);

  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex(prev => Math.min(prev + 1, displayCommands.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex(prev => Math.max(prev - 1, 0));
    } else if (e.key === 'Enter' && displayCommands[selectedIndex]) {
      e.preventDefault();
      handleSelect(displayCommands[selectedIndex]);
    } else if (e.key === 'Escape') {
      onClose();
    }
  }, [displayCommands, selectedIndex, handleSelect, onClose]);

  if (!open) return null;

  let lastGroup = '';

  return (
    <div className="cmd-palette-overlay" onClick={onClose}>
      <div className="cmd-palette" onClick={e => e.stopPropagation()}>
        <div className="cmd-palette-input-row">
          <Search size={16} className="cmd-palette-search-icon" />
          <input
            ref={inputRef}
            type="text"
            className="cmd-palette-input"
            placeholder="Type a command..."
            value={query}
            onChange={e => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
          />
        </div>
        <div className="cmd-palette-list">
          {displayCommands.length === 0 && (
            <div className="cmd-palette-empty">No matching commands</div>
          )}
          {displayCommands.map((cmd, i) => {
            const showGroup = cmd.group && cmd.group !== lastGroup;
            lastGroup = cmd.group ?? '';
            return (
              <div key={cmd.id}>
                {showGroup && <div className="cmd-palette-group">{cmd.group}</div>}
                <button
                  className={`cmd-palette-item ${i === selectedIndex ? 'selected' : ''}`}
                  onClick={() => handleSelect(cmd)}
                  onMouseEnter={() => setSelectedIndex(i)}
                >
                  <span className="cmd-item-icon">{cmd.icon}</span>
                  <span className="cmd-item-label">{cmd.label}</span>
                  {cmd.shortcut && <span className="cmd-item-shortcut">{cmd.shortcut}</span>}
                </button>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
