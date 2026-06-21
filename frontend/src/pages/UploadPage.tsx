import { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { listUploads, uploadFile, type UploadedFile } from '../api/client';
import './UploadPage.css';

type Status = 'idle' | 'uploading' | 'done' | 'error';

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1_048_576) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1_048_576).toFixed(1)} MB`;
}

export function UploadPage() {
  const inputRef = useRef<HTMLInputElement>(null);
  const [file, setFile] = useState<File | null>(null);
  const [status, setStatus] = useState<Status>('idle');
  const [message, setMessage] = useState('');
  const [dragOver, setDragOver] = useState(false);
  const [uploads, setUploads] = useState<UploadedFile[]>([]);
  const navigate = useNavigate();

  useEffect(() => { void fetchUploads(); }, []);

  async function fetchUploads() {
    setUploads(await listUploads());
  }

  function pickFile(f: File) {
    setFile(f);
    setStatus('idle');
    setMessage('');
  }

  function handleChange(e: React.ChangeEvent<HTMLInputElement>) {
    const f = e.target.files?.[0];
    if (f) pickFile(f);
  }

  function handleDragOver(e: React.DragEvent) {
    e.preventDefault();
    setDragOver(true);
  }

  function handleDragLeave() {
    setDragOver(false);
  }

  function handleDrop(e: React.DragEvent) {
    e.preventDefault();
    setDragOver(false);
    const f = e.dataTransfer.files[0];
    if (f) pickFile(f);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!file) return;
    setStatus('uploading');
    const result = await uploadFile(file).catch(() => null);
    if (result) {
      setStatus('done');
      setMessage(`Uploaded: ${result.filename} (${formatSize(result.size)})`);
      setFile(null);
      if (inputRef.current) inputRef.current.value = '';
      void fetchUploads();
    } else {
      setStatus('error');
      setMessage('Upload failed — check the server.');
    }
  }

  return (
    <div className="upload-page">
      <header className="upload-header">
        <h1>Upload File</h1>
        <button type="button" onClick={() => navigate('/dashboard')}>
          Back to Dashboard
        </button>
      </header>
      <main>
        <form className="upload-form" onSubmit={handleSubmit}>
          <label
            className={`upload-drop-zone${dragOver ? ' upload-drop-zone--active' : ''}`}
            htmlFor="file-input"
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
          >
            {file ? (
              <span>
                {file.name} <span className="upload-file-size">{formatSize(file.size)}</span>
              </span>
            ) : (
              'Click or drag a file here'
            )}
            <input
              id="file-input"
              ref={inputRef}
              type="file"
              onChange={handleChange}
            />
          </label>
          <button
            type="submit"
            disabled={!file || status === 'uploading'}
            className="upload-btn"
          >
            {status === 'uploading' ? 'Uploading…' : 'Upload'}
          </button>
          {message && (
            <p className={`upload-msg upload-msg--${status}`}>{message}</p>
          )}
        </form>

        {uploads.length > 0 && (
          <section className="upload-history">
            <h2>Uploaded files</h2>
            <ul>
              {uploads.map((f) => (
                <li key={f.filename}>
                  <span className="upload-history-name">{f.filename}</span>
                  <span className="upload-file-size">{formatSize(f.size)}</span>
                </li>
              ))}
            </ul>
          </section>
        )}
      </main>
    </div>
  );
}
