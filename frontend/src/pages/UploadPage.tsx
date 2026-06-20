import { useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import './UploadPage.css';

type Status = 'idle' | 'uploading' | 'done' | 'error';

export function UploadPage() {
  const inputRef = useRef<HTMLInputElement>(null);
  const [file, setFile] = useState<File | null>(null);
  const [status, setStatus] = useState<Status>('idle');
  const [message, setMessage] = useState('');
  const navigate = useNavigate();

  function handleChange(e: React.ChangeEvent<HTMLInputElement>) {
    setFile(e.target.files?.[0] ?? null);
    setStatus('idle');
    setMessage('');
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!file) return;

    const body = new FormData();
    body.append('file', file);

    setStatus('uploading');
    try {
      const res = await fetch('/api/upload', {
        method: 'POST',
        credentials: 'include',
        body,
      });
      if (res.ok) {
        setStatus('done');
        setMessage('File uploaded successfully.');
        setFile(null);
        if (inputRef.current) inputRef.current.value = '';
      } else {
        const text = await res.text();
        setStatus('error');
        setMessage(text || `Upload failed (${res.status})`);
      }
    } catch {
      setStatus('error');
      setMessage('Network error — could not reach the server.');
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
          <label className="upload-drop-zone" htmlFor="file-input">
            {file ? file.name : 'Click or drag a file here'}
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
      </main>
    </div>
  );
}
