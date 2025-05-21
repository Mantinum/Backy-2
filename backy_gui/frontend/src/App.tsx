import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';

function App() {
  const [source, setSource] = useState<string>('');
  const [dest, setDest] = useState<string>('');
  const [output, setOutput] = useState<string>('');
  const [chunkCount, setChunkCount] = useState<number | null>(null);
  const [loading, setLoading] = useState<boolean>(false);

  // Local backup using native command
  const handleLocalBackup = async () => {
    if (!source || !dest) {
      setOutput('Veuillez spécifier un chemin source et un répertoire de destination.');
      return;
    }
    setLoading(true);
    setOutput('');
    try {
interface SaveBlobLocalArgs extends Record<string, unknown> {
  path: string;
  destDir: string;
}

      const args: SaveBlobLocalArgs = {
        path: source,
        destDir: dest
      };
      const res: string = await invoke('save_blob_local_cmd', args);
      setOutput(`Sauvegarde locale écrite : ${res}`);
    } catch (err) {
      setOutput(String(err));
    } finally {
      setLoading(false);
    }
  };

  // Remote backup (core) via kopia
  const handleBackup = async () => {
    if (!source) {
      setOutput('Veuillez spécifier un dossier à sauvegarder.');
      return;
    }
    setLoading(true);
    setOutput('');
    try {
      const res: string = await invoke('backup_start_cmd', { source });
      setOutput(res);
    } catch (err) {
      setOutput(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleChunk = async () => {
    if (!source) {
      setOutput('Veuillez spécifier un chemin de fichier pour le découpage.');
      return;
    }
    setLoading(true);
    setOutput('');
    setChunkCount(null);
    try {
      const count: number = await invoke('chunk_file_cmd', { path: source });
      setChunkCount(count);
    } catch (err) {
      setOutput(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="container">
      <h1 className="text-3xl font-bold text-center mb-6">Backy 2.0</h1>
      <label className="block text-sm font-medium mb-1">Chemin du fichier ou dossier :</label>
      <div className="flex mb-4">
        <input
          type="text"
          value={source}
          onChange={(e) => setSource(e.currentTarget.value)}
          className="border rounded px-3 py-2 w-full max-w-md mr-2"
          placeholder="Sélectionnez un fichier ou dossier"
        />
        <div className="flex">
          <button
            onClick={async () => {
              const selected = await invoke('open_file_dialog');
              if (selected) {
                setSource(selected as string);
              }
            }}
            className="bg-blue-500 hover:bg-blue-600 text-white font-medium px-4 py-2 rounded-l border-r border-blue-600 flex items-center"
          >
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" className="w-5 h-5 mr-2">
              <path fillRule="evenodd" d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM6 2a2 2 0 00-2 2v12a2 2 0 002 2h8a2 2 0 002-2V7.414A2 2 0 0015.414 6L12 2.586A2 2 0 0010.586 2H6z" clipRule="evenodd" />
            </svg>
            Fichier
          </button>
          <button
            onClick={async () => {
              const selected = await invoke('open_directory_dialog');
              if (selected) {
                setSource(selected as string);
              }
            }}
            className="bg-blue-500 hover:bg-blue-600 text-white font-medium px-4 py-2 rounded-r flex items-center"
          >
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" className="w-5 h-5 mr-2">
              <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
            </svg>
            Dossier
          </button>
        </div>
      </div>
      <label className="block text-sm font-medium mb-1">Répertoire de destination (local) :</label>
      <div className="flex mb-4">
        <input
          type="text"
          value={dest}
          onChange={(e) => setDest(e.currentTarget.value)}
          className="border rounded px-3 py-2 w-full max-w-md mr-2"
          placeholder="Sélectionnez un répertoire de destination"
        />
        <button
          onClick={async () => {
            const selected = await invoke('open_directory_dialog');
            if (selected) {
              setDest(selected as string);
            }
          }}
          className="bg-blue-500 hover:bg-blue-600 text-white font-medium px-4 py-2 rounded flex items-center"
        >
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" className="w-5 h-5 mr-2">
            <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
          </svg>
          Parcourir
        </button>
      </div>
      <div className="flex space-x-4 mb-4">
        <button
          onClick={handleLocalBackup}
          disabled={loading}
          className="button button-primary"
        >
          {loading ? (
            <svg className="animate-spin h-5 w-5 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
          ) : (
            <svg className="h-5 w-5 mr-2" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
              <path d="M10.707 2.293a1 1 0 00-1.414 0l-7 7a1 1 0 001.414 1.414L4 10.414V17a1 1 0 001 1h2a1 1 0 001-1v-2a1 1 0 011-1h2a1 1 0 011 1v2a1 1 0 001 1h2a1 1 0 001-1v-6.586l.293.293a1 1 0 001.414-1.414l-7-7z" />
            </svg>
          )}
          {loading ? 'Sauvegarde locale...' : 'Sauvegarde locale'}
        </button>
        <button
          onClick={handleBackup}
          disabled={loading}
          className="button button-primary"
        >
          {loading ? (
            <svg className="animate-spin h-5 w-5 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
          ) : (
            <svg className="h-5 w-5 mr-2" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
              <path d="M10.707 2.293a1 1 0 00-1.414 0l-7 7a1 1 0 001.414 1.414L4 10.414V17a1 1 0 001 1h2a1 1 0 001-1v-2a1 1 0 011-1h2a1 1 0 011 1v2a1 1 0 001 1h2a1 1 0 001-1v-6.586l.293.293a1 1 0 001.414-1.414l-7-7z" />
            </svg>
          )}
          {loading ? 'Sauvegarde distante...' : 'Sauvegarde distante'}
        </button>
        <button
          onClick={handleChunk}
          disabled={loading}
          className="button button-primary"
        >
          {loading ? (
            <svg className="animate-spin h-5 w-5 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
          ) : (
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" className="h-5 w-5 mr-2">
              <path d="M7 3a1 1 0 000 2h6a1 1 0 100-2H7zM4 7a1 1 0 011-1h10a1 1 0 110 2H5a1 1 0 01-1-1zM2 11a2 2 0 012-2h12a2 2 0 012 2v4a2 2 0 01-2 2H4a2 2 0 01-2-2v-4z" />
            </svg>
          )}
          {loading ? 'Analyse chunks...' : 'Nombre de chunks'}
        </button>
      </div>
      {output && (
        <div className="w-full max-w-md bg-white rounded shadow p-4 overflow-auto mb-4">
          <pre className="whitespace-pre-wrap text-sm">{output}</pre>
        </div>
      )}
      {chunkCount !== null && (
        <div className="w-full max-w-md bg-white rounded shadow p-4">
          <p className="text-lg">Nombre de chunks : {chunkCount}</p>
        </div>
      )}
    </div>
  );
}

export default App;
