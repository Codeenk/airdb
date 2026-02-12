import { useRef, useEffect, useCallback } from 'react';
import { EditorView, keymap, placeholder as cmPlaceholder, lineNumbers, highlightActiveLineGutter, highlightSpecialChars, drawSelection, dropCursor, rectangularSelection, crosshairCursor, highlightActiveLine } from '@codemirror/view';
import { EditorState, Compartment } from '@codemirror/state';
import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands';
import { sql, SQLite, PostgreSQL, MySQL, SQLDialect } from '@codemirror/lang-sql';
import { autocompletion, completionKeymap, acceptCompletion } from '@codemirror/autocomplete';
import { syntaxHighlighting, defaultHighlightStyle, bracketMatching, foldGutter, foldKeymap, indentOnInput, HighlightStyle } from '@codemirror/language';
import { searchKeymap, highlightSelectionMatches } from '@codemirror/search';
import { tags } from '@lezer/highlight';

/* ─── Void Cyan theme override ─── */
const voidCyanTheme = EditorView.theme({
  '&': {
    backgroundColor: 'var(--surface-0)',
    color: 'var(--text-primary)',
    fontSize: '13px',
    fontFamily: 'var(--font-mono)',
  },
  '.cm-content': {
    caretColor: 'var(--accent)',
    padding: '8px 0',
  },
  '.cm-cursor, .cm-dropCursor': {
    borderLeftColor: 'var(--accent)',
    borderLeftWidth: '2px',
  },
  '&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection': {
    backgroundColor: 'rgba(0, 212, 170, 0.15)',
  },
  '.cm-panels': {
    backgroundColor: 'var(--surface-1)',
    color: 'var(--text-primary)',
  },
  '.cm-panels.cm-panels-top': {
    borderBottom: '1px solid var(--surface-3)',
  },
  '.cm-searchMatch': {
    backgroundColor: 'rgba(0, 212, 170, 0.3)',
    borderRadius: '2px',
  },
  '.cm-searchMatch.cm-searchMatch-selected': {
    backgroundColor: 'rgba(0, 212, 170, 0.5)',
  },
  '.cm-activeLine': {
    backgroundColor: 'rgba(255, 255, 255, 0.03)',
  },
  '.cm-selectionMatch': {
    backgroundColor: 'rgba(0, 212, 170, 0.1)',
  },
  '.cm-matchingBracket, .cm-nonmatchingBracket': {
    backgroundColor: 'rgba(0, 212, 170, 0.2)',
    outline: '1px solid rgba(0, 212, 170, 0.4)',
  },
  '.cm-gutters': {
    backgroundColor: 'var(--surface-0)',
    color: 'var(--text-tertiary)',
    border: 'none',
    borderRight: '1px solid var(--surface-2)',
  },
  '.cm-activeLineGutter': {
    backgroundColor: 'rgba(255, 255, 255, 0.03)',
    color: 'var(--text-secondary)',
  },
  '.cm-foldPlaceholder': {
    backgroundColor: 'var(--surface-2)',
    color: 'var(--text-secondary)',
    border: 'none',
  },
  '.cm-tooltip': {
    backgroundColor: 'var(--surface-1)',
    border: '1px solid var(--surface-3)',
    borderRadius: '6px',
    boxShadow: '0 4px 12px rgba(0,0,0,0.4)',
  },
  '.cm-tooltip .cm-tooltip-arrow:before': {
    borderTopColor: 'var(--surface-3)',
  },
  '.cm-tooltip .cm-tooltip-arrow:after': {
    borderTopColor: 'var(--surface-1)',
  },
  '.cm-tooltip-autocomplete': {
    '& > ul > li[aria-selected]': {
      backgroundColor: 'rgba(0, 212, 170, 0.15)',
      color: 'var(--text-primary)',
    },
  },
  '.cm-completionLabel': {
    color: 'var(--text-primary)',
  },
  '.cm-completionDetail': {
    color: 'var(--text-tertiary)',
    fontStyle: 'italic',
  },
  '.cm-completionMatchedText': {
    color: 'var(--accent)',
    textDecoration: 'none',
    fontWeight: 600,
  },
}, { dark: true });

const voidCyanHighlight = HighlightStyle.define([
  { tag: tags.keyword, color: '#c792ea', fontWeight: '600' },
  { tag: tags.operator, color: '#89ddff' },
  { tag: tags.special(tags.string), color: '#f07178' },
  { tag: tags.string, color: '#c3e88d' },
  { tag: tags.number, color: '#f78c6c' },
  { tag: tags.bool, color: '#ff5370' },
  { tag: tags.null, color: '#ff5370', fontStyle: 'italic' },
  { tag: tags.comment, color: '#546e7a', fontStyle: 'italic' },
  { tag: tags.typeName, color: '#ffcb6b' },
  { tag: tags.standard(tags.name), color: '#82aaff' },
  { tag: tags.definition(tags.name), color: '#82aaff' },
  { tag: tags.name, color: '#eeffff' },
  { tag: tags.punctuation, color: '#89ddff' },
  { tag: tags.paren, color: '#89ddff' },
  { tag: tags.squareBracket, color: '#89ddff' },
  { tag: tags.brace, color: '#89ddff' },
]);

/* ─── Dialect map ─── */
function getSqlDialect(dialectName: string): SQLDialect {
  switch (dialectName.toLowerCase()) {
    case 'postgres': return PostgreSQL;
    case 'mysql': return MySQL;
    default: return SQLite;
  }
}

/* ─── Props ─── */
interface SqlEditorProps {
  value: string;
  onChange: (value: string) => void;
  onExecute?: () => void;
  dialect?: string;
  tables?: Record<string, string[]>; // table → column names for autocomplete
  placeholder?: string;
  readOnly?: boolean;
  minHeight?: string;
}

export function SqlEditor({ value, onChange, onExecute, dialect = 'sqlite', tables, placeholder = 'SELECT * FROM users;', readOnly = false, minHeight = '200px' }: SqlEditorProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const dialectComp = useRef(new Compartment());
  const readOnlyComp = useRef(new Compartment());
  const onChangeRef = useRef(onChange);
  const onExecuteRef = useRef(onExecute);
  onChangeRef.current = onChange;
  onExecuteRef.current = onExecute;

  const buildSqlExtension = useCallback((d: string, t?: Record<string, string[]>) => {
    const schema: Record<string, string[]> = t || {};
    return sql({
      dialect: getSqlDialect(d),
      schema,
      upperCaseKeywords: true,
    });
  }, []);

  useEffect(() => {
    if (!containerRef.current) return;

    const executeKeymap = keymap.of([{
      key: 'Ctrl-Enter',
      mac: 'Cmd-Enter',
      run: () => {
        onExecuteRef.current?.();
        return true;
      },
    }, {
      key: 'Tab',
      run: acceptCompletion,
    }]);

    const startState = EditorState.create({
      doc: value,
      extensions: [
        lineNumbers(),
        highlightActiveLineGutter(),
        highlightSpecialChars(),
        history(),
        foldGutter(),
        drawSelection(),
        dropCursor(),
        EditorState.allowMultipleSelections.of(true),
        indentOnInput(),
        bracketMatching(),
        rectangularSelection(),
        crosshairCursor(),
        highlightActiveLine(),
        highlightSelectionMatches(),
        autocompletion({ defaultKeymap: true }),
        executeKeymap,
        keymap.of([
          ...defaultKeymap,
          ...historyKeymap,
          ...foldKeymap,
          ...searchKeymap,
          ...completionKeymap,
          indentWithTab,
        ]),
        dialectComp.current.of(buildSqlExtension(dialect, tables)),
        readOnlyComp.current.of(EditorState.readOnly.of(readOnly)),
        voidCyanTheme,
        syntaxHighlighting(voidCyanHighlight),
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
        cmPlaceholder(placeholder),
        EditorView.updateListener.of(update => {
          if (update.docChanged) {
            onChangeRef.current(update.state.doc.toString());
          }
        }),
        EditorView.lineWrapping,
      ],
    });

    const view = new EditorView({
      state: startState,
      parent: containerRef.current,
    });

    viewRef.current = view;
    return () => { view.destroy(); };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Update dialect/tables when they change
  useEffect(() => {
    if (viewRef.current) {
      viewRef.current.dispatch({
        effects: dialectComp.current.reconfigure(buildSqlExtension(dialect, tables)),
      });
    }
  }, [dialect, tables, buildSqlExtension]);

  // Update readOnly
  useEffect(() => {
    if (viewRef.current) {
      viewRef.current.dispatch({
        effects: readOnlyComp.current.reconfigure(EditorState.readOnly.of(readOnly)),
      });
    }
  }, [readOnly]);

  // Sync value from outside (e.g., loading saved query)
  useEffect(() => {
    const view = viewRef.current;
    if (view && value !== view.state.doc.toString()) {
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: value },
      });
    }
  }, [value]);

  return (
    <div
      ref={containerRef}
      className="sql-editor-cm"
      style={{ flex: 1, minHeight, overflow: 'auto', border: '1px solid var(--surface-3)', borderRadius: '6px' }}
    />
  );
}
