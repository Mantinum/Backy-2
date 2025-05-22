import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';

function App() {
  const [source, setSource] = useState<string>('');
  const [dest, setDest] = useState<string>('');
  const [output, setOutput] = useState<string>('');
  const [chunkCount, setChunkCount] = useState<number | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  const [progress, setProgress] = useState<number>(0);
  const [sftpHost, setSftpHost] = useState<string>('');
  const [sftpPort, setSftpPort] = useState<number>(22);
  const [sftpUsername, setSftpUsername] = useState<string>('');
  const [sftpPassword, setSftpPassword] = useState<string>('');
  const [sftpRemotePath, setSftpRemotePath] = useState<string>('');

  // Local backup using native command
  const handleLocalBackup = async () => {
      if (!source || !dest) {
        setOutput('Veuillez spécifier un chemin source et un répertoire de sauvegarde locale.');
      return;
    }
    setLoading(true);
    setOutput('');
    setProgress(0);
    try {
      interface SaveBlobLocalArgs extends Record<string, unknown> {
        path: string;
        destDir: string;
      }

      const args: SaveBlobLocalArgs = {
        path: source,
        destDir: dest
      };
      
      // Simulate progress (replace with actual progress updates)
      const interval = setInterval(() => {
        setProgress(prev => Math.min(prev + 10, 90));
      }, 300);

      const res: string = await invoke('save_blob_local_cmd', args);
      clearInterval(interval);
      setProgress(100);
      setOutput(`Sauvegarde locale réussie : ${res}`);
    } catch (err) {
      setOutput(`Erreur : ${String(err)}`);
    } finally {
      setLoading(false);
    }
  };

  const handleSftpBackup = async () => {
    if (!source) {
      setOutput('Veuillez spécifier un fichier ou dossier source');
      return;
    }
    if (!sftpHost || !sftpUsername || !sftpPassword || !sftpRemotePath) {
      setOutput('Veuillez remplir tous les champs de configuration SFTP');
      return;
    }
    setLoading(true);
    setOutput('');
    try {
      interface SftpBackupArgs extends Record<string, unknown> {
        host: string;
        port: number;
        username: string;
        password: string;
        localPath: string;
        remotePath: string;
      }

      const result = await invoke('sftp_backup', {
          host: sftpHost,
          port: sftpPort,
          username: sftpUsername,
          password: sftpPassword,
          localPath: source,
          remotePath: sftpRemotePath
        });
      setOutput(String(result));
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
      <h1>Backy 2.0</h1>

      {/* Source Section */}
      <div className="section" style={{ backgroundColor: 'var(--section-source)' }}>
        <label>Chemin du fichier ou dossier</label>
        <div className="input-group">
          <input
            type="text"
            value={source}
            onChange={(e) => setSource(e.currentTarget.value)}
            className="input"
            placeholder="Sélectionnez un fichier ou dossier"
          />
          <div className="button-group">
            <button
              onClick={async () => {
                const selected = await invoke('open_file_dialog');
                if (selected) {
                  setSource(selected as string);
                }
              }}
              className="button"
            >
              Fichier
            </button>
            <button
              onClick={async () => {
                const selected = await invoke('open_directory_dialog');
                if (selected) {
                  setSource(selected as string);
                }
              }}
              className="button"
            >
              Dossier
            </button>
          </div>
        </div>
      </div>

      {/* Destination Section */}
      <div className="section" style={{ backgroundColor: 'var(--section-dest)' }}>
        <label>Répertoire de sauvegarde locale</label>
        <div className="input-group">
          <input
            type="text"
            value={dest}
            onChange={(e) => setDest(e.currentTarget.value)}
            className="input"
            placeholder="Sélectionnez un répertoire de destination"
          />
          <button
            onClick={async () => {
              const selected = await invoke('open_directory_dialog');
              if (selected) {
                setDest(selected as string);
              }
            }}
            className="button"
          >
            Parcourir
          </button>
        </div>
      </div>

      {/* Progress Bar */}
      {loading && (
        <div className="section" style={{ backgroundColor: 'var(--section-actions)' }}>
          <div className="progress-bar">
            <div
              className="progress"
              style={{ width: `${progress}%` }}
            ></div>
          </div>
        </div>
      )}

      {/* SFTP Configuration Section */}
      <div className="section" style={{ backgroundColor: 'var(--section-actions)' }}>
        <h2>Configuration SFTP</h2>
        <div className="input-group">
          <input
            type="text"
            placeholder="Hôte"
            value={sftpHost}
            onChange={(e) => setSftpHost(e.target.value)}
            className="input"
          />
          <input
            type="number"
            placeholder="Port"
            value={sftpPort}
            onChange={(e) => setSftpPort(Number(e.target.value))}
            className="input"
          />
        </div>
        <div className="input-group">
          <input
            type="text"
            placeholder="Nom d'utilisateur"
            value={sftpUsername}
            onChange={(e) => setSftpUsername(e.target.value)}
            className="input"
          />
          <input
            type="password"
            placeholder="Mot de passe"
            value={sftpPassword}
            onChange={(e) => setSftpPassword(e.target.value)}
            className="input"
          />
        </div>
        <div className="input-group">
          <input
            type="text"
            placeholder="Chemin distant"
            value={sftpRemotePath}
            onChange={(e) => setSftpRemotePath(e.target.value)}
            className="input"
          />
        </div>
      </div>

      {/* Actions Section */}
      <div className="section" style={{ backgroundColor: 'var(--section-actions)' }}>
        <div className="button-group">
          <button
            onClick={handleLocalBackup}
            disabled={loading}
            className="button"
          >
            {loading ? (
              <span className="loading">⏳</span>
            ) : (
              'Sauvegarde locale'
            )}
          </button>
          <button
            onClick={handleSftpBackup}
            disabled={loading}
            className="button"
          >
            {loading ? (
              <span className="loading">⏳</span>
            ) : (
              'Sauvegarde SFTP'
            )}
          </button>
          <button
            onClick={handleChunk}
            disabled={loading}
            className="button"
          >
            {loading ? (
              <span className="loading">⏳</span>
            ) : (
              'Nombre de chunks'
            )}
          </button>
        </div>
      </div>

      {/* Output Section */}
      {output && (
        <div className="section output" style={{ backgroundColor: 'var(--section-output)' }}>
          <pre>{output}</pre>
        </div>
      )}

      {/* Chunk Count Section */}
      {chunkCount !== null && (
        <div className="section output">
          <p>Nombre de chunks : {chunkCount}</p>
        </div>
      )}
    </div>
  );
}

export default App;
