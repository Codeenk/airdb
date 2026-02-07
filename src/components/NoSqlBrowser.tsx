import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './NoSqlBrowser.css';

interface Collection {
    name: string;
    count?: number;
}

interface Document {
    _id: string;
    _schema_version?: number;
    _created_at: string;
    _modified_at: string;
    data: any;
}

export function NoSqlBrowser() {
    const [collections, setCollections] = useState<Collection[]>([]);
    const [selectedCollection, setSelectedCollection] = useState<string | null>(null);
    const [documents, setDocuments] = useState<Document[]>([]);
    const [newCollectionName, setNewCollectionName] = useState('');
    const [newDocData, setNewDocData] = useState('{}');
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        loadCollections();
    }, []);

    const loadCollections = async () => {
        try {
            setLoading(true);
            const cols = await invoke<string[]>('nosql_list_collections');
            setCollections(cols.map(name => ({ name })));
            setError(null);
        } catch (err) {
            setError(`Failed to load collections: ${err}`);
        } finally {
            setLoading(false);
        }
    };

    const createCollection = async () => {
        if (!newCollectionName.trim()) return;

        try {
            setLoading(true);
            await invoke('nosql_create_collection', { name: newCollectionName });
            setNewCollectionName('');
            await loadCollections();
            setError(null);
        } catch (err) {
            setError(`Failed to create collection: ${err}`);
        } finally {
            setLoading(false);
        }
    };

    const loadDocuments = async (collectionName: string) => {
        try {
            setLoading(true);
            setSelectedCollection(collectionName);
            const docs = await invoke<any[]>('nosql_query', {
                collection: collectionName,
                filters: [],
                limit: 100
            });
            setDocuments(docs);
            setError(null);
        } catch (err) {
            setError(`Failed to load documents: ${err}`);
        } finally {
            setLoading(false);
        }
    };

    const insertDocument = async () => {
        if (!selectedCollection) return;

        try {
            setLoading(true);
            const data = JSON.parse(newDocData);
            await invoke('nosql_insert', {
                collection: selectedCollection,
                data
            });
            setNewDocData('{}');
            await loadDocuments(selectedCollection);
            setError(null);
        } catch (err) {
            setError(`Failed to insert document: ${err}`);
        } finally {
            setLoading(false);
        }
    };

    const deleteDocument = async (id: string) => {
        if (!selectedCollection) return;

        try {
            setLoading(true);
            await invoke('nosql_delete', {
                collection: selectedCollection,
                id
            });
            await loadDocuments(selectedCollection);
            setError(null);
        } catch (err) {
            setError(`Failed to delete document: ${err}`);
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="nosql-browser">
            <div className="sidebar">
                <h2>Collections</h2>

                <div className="create-collection">
                    <input
                        type="text"
                        value={newCollectionName}
                        onChange={(e) => setNewCollectionName(e.target.value)}
                        placeholder="Collection name"
                        disabled={loading}
                    />
                    <button onClick={createCollection} disabled={loading || !newCollectionName.trim()}>
                        Create
                    </button>
                </div>

                <div className="collection-list">
                    {collections.map(col => (
                        <div
                            key={col.name}
                            className={`collection-item ${selectedCollection === col.name ? 'active' : ''}`}
                            onClick={() => loadDocuments(col.name)}
                        >
                            {col.name}
                        </div>
                    ))}
                </div>
            </div>

            <div className="main-content">
                {error && (
                    <div className="error-banner">
                        {error}
                        <button onClick={() => setError(null)}>×</button>
                    </div>
                )}

                {selectedCollection ? (
                    <>
                        <div className="collection-header">
                            <h2>{selectedCollection}</h2>
                            <span className="doc-count">{documents.length} documents</span>
                        </div>

                        <div className="insert-form">
                            <textarea
                                value={newDocData}
                                onChange={(e) => setNewDocData(e.target.value)}
                                placeholder='{"field": "value"}'
                                disabled={loading}
                            />
                            <button onClick={insertDocument} disabled={loading}>
                                Insert Document
                            </button>
                        </div>

                        <div className="documents-grid">
                            {documents.map(doc => (
                                <div key={doc._id} className="document-card">
                                    <div className="doc-header">
                                        <code className="doc-id">{doc._id}</code>
                                        <button
                                            className="delete-btn"
                                            onClick={() => deleteDocument(doc._id)}
                                            disabled={loading}
                                        >
                                            Delete
                                        </button>
                                    </div>
                                    <pre className="doc-data">
                                        {JSON.stringify(doc.data, null, 2)}
                                    </pre>
                                    <div className="doc-meta">
                                        <span>Created: {new Date(doc._created_at).toLocaleString()}</span>
                                        <span>Modified: {new Date(doc._modified_at).toLocaleString()}</span>
                                    </div>
                                </div>
                            ))}
                        </div>
                    </>
                ) : (
                    <div className="empty-state">
                        <p>Select a collection to view documents</p>
                    </div>
                )}
            </div>
        </div>
    );
}
