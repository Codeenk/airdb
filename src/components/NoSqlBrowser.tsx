import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
    FileJson,
    Plus,
    Trash2,
    Search,
    RefreshCw,
    Database,
    Copy,
    Info,
    AlertCircle
} from 'lucide-react';
import { Collection, Document } from '../types';
import { ConfirmDialog } from './ConfirmDialog';
import './NoSqlBrowser.css';

interface NoSqlBrowserProps { }

export function NoSqlBrowser({ }: NoSqlBrowserProps) {
    /* ─── State ─── */
    const [collections, setCollections] = useState<Collection[]>([]);
    const [selectedCollection, setSelectedCollection] = useState<string | null>(null);
    const [documents, setDocuments] = useState<Document[]>([]);
    const [loading, setLoading] = useState(false);
    const [searchQuery, setSearchQuery] = useState('');
    const [selectedDocId, setSelectedDocId] = useState<string | null>(null);

    // New/Insert State
    const [isCreatingCollection, setIsCreatingCollection] = useState(false);
    const [newCollectionName, setNewCollectionName] = useState('');
    const [isInsertingDoc, setIsInsertingDoc] = useState(false);
    const [newDocJson, setNewDocJson] = useState('{\n  "key": "value"\n}');
    const [jsonError, setJsonError] = useState<string | null>(null);    const [confirmDelete, setConfirmDelete] = useState<string | null>(null);
    const [insertError, setInsertError] = useState<string | null>(null);
    /* ─── Effects ─── */
    useEffect(() => {
        loadCollections();
    }, []);

    useEffect(() => {
        if (selectedCollection) {
            loadDocuments(selectedCollection);
            setSelectedDocId(null);
        } else {
            setDocuments([]);
        }
    }, [selectedCollection]);

    // Validate JSON on change
    useEffect(() => {
        try {
            JSON.parse(newDocJson);
            setJsonError(null);
        } catch (e) {
            setJsonError((e as Error).message);
        }
    }, [newDocJson]);

    /* ─── Actions ─── */
    async function loadCollections() {
        try {
            const result = await invoke<any>('nosql_list_collections');

            if (Array.isArray(result) && typeof result[0] === 'string') {
                setCollections(result.map(name => ({ name, count: 0, size_bytes: 0 })));
            } else {
                setCollections(result);
            }
        } catch (e) {
            console.error('Failed to load collections', e);
        }
    }

    async function loadDocuments(collectionName: string) {
        setLoading(true);
        try {
            const result = await invoke<any[]>('nosql_query', {
                collection: collectionName,
                filters: [],
                limit: 100
            });

            const adaptedDocs: Document[] = result.map((doc: any) => ({
                id: doc._id || doc.id || 'unknown',
                data: doc,
                created_at: doc.created_at,
                updated_at: doc.updated_at
            }));

            setDocuments(adaptedDocs);
        } catch (e) {
            console.error('Failed to load documents', e);
        } finally {
            setLoading(false);
        }
    }

    async function handleCreateCollection() {
        if (!newCollectionName.trim()) return;
        try {
            await invoke('nosql_create_collection', { name: newCollectionName });
            await loadCollections();
            setSelectedCollection(newCollectionName);
            setIsCreatingCollection(false);
            setNewCollectionName('');
        } catch (e) {
            console.error(e);
        }
    }

    async function handleInsertDocument() {
        if (!selectedCollection || jsonError) return;
        try {
            const data = JSON.parse(newDocJson);
            await invoke('nosql_insert', { collection: selectedCollection, data });
            setIsInsertingDoc(false);
            setInsertError(null);
            loadDocuments(selectedCollection);
            setNewDocJson('{\n  "key": "value"\n}');
        } catch (e) {
            setInsertError(String(e));
        }
    }

    async function handleDeleteDocument(id: string) {
        if (!selectedCollection) return;
        setConfirmDelete(id);
    }

    async function confirmDeleteDocument() {
        if (!selectedCollection || !confirmDelete) return;
        try {
            await invoke('nosql_delete', { collection: selectedCollection, id: confirmDelete });
            loadDocuments(selectedCollection);
            if (selectedDocId === confirmDelete) setSelectedDocId(null);
        } catch (e) {
            console.error(e);
        } finally {
            setConfirmDelete(null);
        }
    }

    const selectedDoc = documents.find(d => d.id === selectedDocId);

    return (
        <div className="nosql-browser-container">
            {/* ─── SIDEBAR: Collections ─── */}
            <div className="ns-sidebar">
                <div className="ns-sidebar-header">
                    <span className="ns-sidebar-title">Collections</span>
                    <button
                        className="btn btn-sm btn-ghost"
                        onClick={() => setIsCreatingCollection(true)}
                        title="New Collection"
                    >
                        <Plus size={16} />
                    </button>
                </div>

                {isCreatingCollection && (
                    <div style={{ padding: '8px' }}>
                        <input
                            className="te-input"
                            autoFocus
                            placeholder="Collection Name"
                            value={newCollectionName}
                            onChange={e => setNewCollectionName(e.target.value)}
                            onKeyDown={e => {
                                if (e.key === 'Enter') handleCreateCollection();
                                if (e.key === 'Escape') setIsCreatingCollection(false);
                            }}
                        />
                    </div>
                )}

                <div className="ns-collection-list">
                    {collections.map(col => (
                        <div
                            key={col.name}
                            className={`ns-collection-item ${selectedCollection === col.name ? 'active' : ''}`}
                            onClick={() => setSelectedCollection(col.name)}
                        >
                            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                                <Database size={14} />
                                <span>{col.name}</span>
                            </div>
                        </div>
                    ))}
                </div>
            </div>

            {/* ─── MAIN: Document List ─── */}
            <div className="ns-main" style={{ borderRight: '1px solid var(--border)' }}>
                {selectedCollection ? (
                    <>
                        <div className="ns-toolbar">
                            <div className="ns-search-bar" style={{ width: '200px' }}>
                                <Search size={14} className="ns-search-icon" />
                                <input
                                    className="ns-search-input"
                                    placeholder="Search..."
                                    value={searchQuery}
                                    onChange={e => setSearchQuery(e.target.value)}
                                />
                            </div>
                            <div style={{ display: 'flex', gap: '8px' }}>
                                <button className="btn btn-ghost" onClick={() => loadDocuments(selectedCollection)} title="Refresh">
                                    <RefreshCw size={14} className={loading ? 'spin' : ''} />
                                </button>
                                <button className="btn btn-primary" onClick={() => setIsInsertingDoc(true)} title="Insert Document">
                                    <Plus size={14} /> Insert
                                </button>
                            </div>
                        </div>

                        <div className="ns-document-list" style={{ padding: 0, gap: 0 }}>
                            {documents
                                .filter(d => JSON.stringify(d).toLowerCase().includes(searchQuery.toLowerCase()))
                                .map(doc => (
                                    <div
                                        key={doc.id}
                                        className={`ns-document-item ${selectedDocId === doc.id ? 'active' : ''}`}
                                        style={{
                                            padding: '12px 16px',
                                            borderBottom: '1px solid var(--border)',
                                            cursor: 'pointer',
                                            background: selectedDocId === doc.id ? 'var(--surface-2)' : 'transparent',
                                            borderLeft: selectedDocId === doc.id ? '3px solid var(--accent)' : '3px solid transparent'
                                        }}
                                        onClick={() => setSelectedDocId(doc.id)}
                                    >
                                        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '4px' }}>
                                            <span style={{ fontFamily: 'var(--font-mono)', fontSize: '11px', color: 'var(--accent)' }}>{doc.id}</span>
                                            <button
                                                className="btn btn-icon btn-ghost btn-danger btn-sm"
                                                onClick={(e) => { e.stopPropagation(); handleDeleteDocument(doc.id); }}
                                                style={{ opacity: 0.5 }}
                                            >
                                                <Trash2 size={12} />
                                            </button>
                                        </div>
                                        <div style={{
                                            fontSize: '12px',
                                            color: 'var(--text-secondary)',
                                            whiteSpace: 'nowrap',
                                            overflow: 'hidden',
                                            textOverflow: 'ellipsis',
                                            fontFamily: 'var(--font-mono)'
                                        }}>
                                            {JSON.stringify(doc.data)}
                                        </div>
                                    </div>
                                ))}

                            {documents.length === 0 && !loading && (
                                <div className="ns-empty-state">
                                    <FileJson size={32} className="ns-empty-icon" />
                                    <p>No documents found</p>
                                </div>
                            )}
                        </div>
                    </>
                ) : (
                    <div className="ns-empty-state">
                        <Database size={48} className="ns-empty-icon" />
                        <h3>Select a collection</h3>
                    </div>
                )}
            </div>

            {/* ─── RIGHT: Document Inspector ─── */}
            <div className="ns-inspector" style={{ width: '400px', background: 'var(--surface-1)', display: 'flex', flexDirection: 'column' }}>
                {selectedDoc ? (
                    <>
                        <div style={{ padding: '16px', borderBottom: '1px solid var(--border)', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                            <span style={{ fontSize: '14px', fontWeight: 600 }}>Document Details</span>
                            <button
                                className="btn btn-icon btn-ghost btn-sm"
                                onClick={() => navigator.clipboard.writeText(JSON.stringify(selectedDoc.data, null, 2))}
                                title="Copy JSON"
                            >
                                <Copy size={14} />
                            </button>
                        </div>

                        <div style={{ flex: 1, overflow: 'auto', padding: '16px' }}>
                            <div style={{
                                background: 'var(--surface-0)',
                                padding: '12px',
                                borderRadius: 'var(--radius-md)',
                                border: '1px solid var(--border)',
                                height: '100%',
                                overflow: 'auto'
                            }}>
                                <pre style={{
                                    fontFamily: 'var(--font-mono)',
                                    fontSize: '12px',
                                    color: 'var(--text-primary)',
                                    margin: 0
                                }}>
                                    {JSON.stringify(selectedDoc.data, null, 2)}
                                </pre>
                            </div>
                        </div>

                        {/* Metadata Panel */}
                        <div style={{ padding: '16px', borderTop: '1px solid var(--border)', background: 'var(--surface-2)' }}>
                            <h4 style={{ fontSize: '11px', textTransform: 'uppercase', color: 'var(--text-tertiary)', marginBottom: '8px' }}>Metadata</h4>
                            <div style={{ display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '8px 16px', fontSize: '12px' }}>
                                <span style={{ color: 'var(--text-secondary)' }}>ID</span>
                                <span style={{ fontFamily: 'var(--font-mono)', color: 'var(--accent)' }}>{selectedDoc.id}</span>

                                <span style={{ color: 'var(--text-secondary)' }}>Created</span>
                                <span style={{ color: 'var(--text-primary)' }}>{selectedDoc.created_at || '-'}</span>

                                <span style={{ color: 'var(--text-secondary)' }}>Updated</span>
                                <span style={{ color: 'var(--text-primary)' }}>{selectedDoc.updated_at || '-'}</span>
                            </div>
                        </div>
                    </>
                ) : (
                    <div className="ns-empty-state">
                        <Info size={24} className="ns-empty-icon" />
                        <p>Select a document to inspect</p>
                    </div>
                )}
            </div>

            {/* ─── INSERT MODAL ─── */}
            {isInsertingDoc && (
                <div className="modal-overlay">
                    <div className="modal" style={{ width: '600px', maxWidth: '90vw' }}>
                        <div className="modal-header">
                            <h3>Insert Document</h3>
                            <button onClick={() => setIsInsertingDoc(false)}><span style={{ fontSize: '20px' }}>×</span></button>
                        </div>
                        <div className="modal-body">
                            <div style={{ position: 'relative' }}>
                                <textarea
                                    className={`input ${jsonError ? 'input-error' : ''}`}
                                    style={{
                                        height: '300px',
                                        fontFamily: 'var(--font-mono)',
                                        fontSize: '13px',
                                        lineHeight: '1.5',
                                        whiteSpace: 'pre',
                                        color: 'var(--text-primary)'
                                    }}
                                    value={newDocJson}
                                    onChange={e => setNewDocJson(e.target.value)}
                                    placeholder="{ ... }"
                                />
                                {jsonError && (
                                    <div style={{
                                        position: 'absolute',
                                        bottom: '12px',
                                        right: '12px',
                                        background: 'rgba(255, 85, 85, 0.9)',
                                        color: 'white',
                                        padding: '4px 8px',
                                        borderRadius: '4px',
                                        fontSize: '11px',
                                        display: 'flex',
                                        alignItems: 'center',
                                        gap: '4px'
                                    }}>
                                        <AlertCircle size={12} /> {jsonError}
                                    </div>
                                )}
                            </div>
                        </div>
                        <div className="modal-footer">
                            <button className="btn btn-ghost" onClick={() => setIsInsertingDoc(false)}>Cancel</button>
                            <button className="btn btn-primary" onClick={handleInsertDocument} disabled={!!jsonError}>
                                {jsonError ? 'Fix Errors' : 'Insert Document'}
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {confirmDelete && (
                <ConfirmDialog
                    isOpen={true}
                    title="Delete Document"
                    message={`Are you sure you want to delete document "${confirmDelete}"? This cannot be undone.`}
                    confirmLabel="Delete"
                    variant="danger"
                    onConfirm={confirmDeleteDocument}
                    onCancel={() => setConfirmDelete(null)}
                />
            )}

            {insertError && (
                <div className="confirm-overlay" onClick={() => setInsertError(null)}>
                    <div className="confirm-dialog" onClick={e => e.stopPropagation()}>
                        <h3 style={{ color: 'var(--danger)' }}>Insert Error</h3>
                        <p>{insertError}</p>
                        <div className="confirm-dialog-actions">
                            <button className="btn btn-primary" onClick={() => setInsertError(null)}>OK</button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
