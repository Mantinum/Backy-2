import { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './App.css';

/**
 * Backy-2 Frontend – main React component.
 * Typé strict ; interfaces utilisées pour éviter les avertissements TS.
 */
function App() {
  /* ======== State ======== */
  const [source, setSource] = useState<string>('');
  const [dest, setDest] = useState<string>('');
  const [output, setOutput] = useState<string>('');
  const [chunkCount, setChunkCount] = useState<number | null>(null);

  const [loading, setLoading]     = useState<boolean>(false);
  const [progress, setProgress]   = useState<number>(0);

  /* SFTP form fields */
  const [sftpHost,        setSftpHost]        = useState<string>('');
  const [sftpPort,        setSftpPort]        = useState<number>(22);
  const [sftpUsername,    setSftpUsername]    = useState<string>('');
  const [sftpPassword,    setSftpPassword]    = useState<string>('');
  const [sftpRemotePath,  setSftpRemotePath]  = useState<string>('');

  /* ======== Types ======== */
  interface SaveBlobLocalArgs {
    path: string;
    destDir: string;
  }

  interface SftpBackupArgs {
    host: string;
    port: number;
    username: string;
    password: string;
    localPath: string;
    remotePath: string;
  }

  /* ======== Helpers ======== */
  const fakeProgress = () => {
    setProgress(0);
    const id = setInterval(() => {
      setProgress(prev => {
        const nxt = prev + 5;
        if (nxt >= 100) {
          clearInterval(id);
          return 100;
        }
        return nxt;
      });
    }, 150);
  };

  /* ======== Actions ======== */
  const handleLocalBackup = async () => {
    if (!source || !dest) {
      setOutput('Veuillez spécifier un chemin source ET une destination locale.');
      return;
    }
    setLoading(true);
    setOutput('');
    fakeProgress();

    const args: SaveBlobLocalArgs = { path: source, destDir: dest };

    try {
      const res: string = await invoke('save_blob_local', { args });
      setOutput(`Sauvegarde locale réussie : ${res}`);
    } catch (err) {
      setOutput(`Erreur : ${String(err)}`);
    } finally {
      setLoading(false);
    }
  };

  const handleSftpBackup = async () => {
    if (!source) {
      setOutput('Veuillez sélectionner un fichier ou dossier source.');
      return;
    }
    if (!sftpHost || !sftpUsername || !sftpPassword || !sftpRemotePath) {
      setOutput('Veuillez remplir tous les champs SFTP.');
      return;
    }
    setLoading(true);
    setOutput('');
    fakeProgress();

    const sftpArgs: SftpBackupArgs = {
      host: sftpHost,
      port: sftpPort,
      username: sftpUsername,
      password: sftpPassword,
      localPath: source,
      remotePath: sftpRemotePath,
    };

    try {
      const res = await invoke('sftp_backup', { args: sftpArgs });
      setOutput(String(res));
    } catch (err) {
      setOutput(`Erreur : ${String(err)}`);
    } finally {
      setLoading(false);
    }
  };

  const handleChunk = async () => {
    if (!source) {
      setOutput('Veuillez spécifier un fichier à découper.');
      return;
    }
    setLoading(true);
    setOutput('');
    fakeProgress();

    try {
      const cnt: number = await invoke('chunk_file_cmd', { path: source });
      setChunkCount(cnt);
      setOutput(`${cnt} morceaux créés.`);
    } catch (err) {
      setOutput(`Erreur : ${String(err)}`);
    } finally {
      setLoading(false);
    }
  };

  /* ======== UI ======== */
  return (
    <div className="container">
      {/* Source selector */}
      <section className="section">
        <label>Chemin source</label>
        <div className="input-group">
          <input
            className="input"
            value={source}
            onChange={(e) => setSource(e.currentTarget.value)}
            placeholder="Choisir fichier ou dossier…"
          />
          <button
            className="button"
            onClick={async () => {
              const selected = await invoke<string | null>('open_file_dialog');
              if (selected) setSource(selected);
            }}
          >
            Parcourir
          </button>
        </div>
      </section>

      {/* Destination selector */}
      <section className="section">
        <label>Dossier cible (sauvegarde locale)</label>
        <div className="input-group">
          <input
            className="input"
            value={dest}
            onChange={(e) => setDest(e.currentTarget.value)}
            placeholder="/chemin/vers/destination" />
          <button
            className="button"
            onClick={async () => {
              const selected = await invoke<string | null>('open_directory_dialog');
              if (selected) setDest(selected);
            }}
          >
            Parcourir
          </button>
        </div>
      </section>

      {/* SFTP configuration */}
      <section className="section">
        <h3>SFTP</h3>
        <div className="input-group">
          <input className="input" placeholder="Hôte" value={sftpHost} onChange={(e)=>setSftpHost(e.target.value)} />
          <input className="input" type="number" placeholder="Port" value={sftpPort} onChange={(e)=>setSftpPort(Number(e.target.value))} />
        </div>
        <div className="input-group">
          <input className="input" placeholder="Utilisateur" value={sftpUsername} onChange={(e)=>setSftpUsername(e.target.value)} />
          <input className="input" type="password" placeholder="Mot de passe" value={sftpPassword} onChange={(e)=>setSftpPassword(e.target.value)} />
        </div>
        <input className="input" placeholder="Chemin distant" value={sftpRemotePath} onChange={(e)=>setSftpRemotePath(e.target.value)} />
      </section>

      {/* Progress bar */}
      {loading && (
        <section className="section">
          <div className="progress-bar" style={{ width: `${progress}%` }} />
        </section>
      )}

      {/* Action buttons */}
      <section className="section actions">
        <button className="button" disabled={loading} onClick={handleLocalBackup}>
          Sauvegarde locale
        </button>
        <button className="button" disabled={loading} onClick={handleSftpBackup}>
          Sauvegarde SFTP
        </button>
        <button className="button" disabled={loading} onClick={handleChunk}>
          Découper en blocs
        </button>
      </section>

      {/* Output */}
      <section className="section output">
        {chunkCount !== null && <p>{chunkCount} morceaux générés.</p>}
        {output && <pre className="terminal">{output}</pre>}
      </section>
    </div>
  );
}

export default App;
