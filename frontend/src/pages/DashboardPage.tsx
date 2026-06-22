import { useNavigate, Link } from 'react-router-dom';
import { logout } from '../api/client';
import { useAuth } from '../auth/useAuth';
import { GraphViewPlaceholder } from '../components/GraphViewPlaceholder';
import './DashboardPage.css';

export function DashboardPage() {
  const { refresh } = useAuth();
  const navigate = useNavigate();

  async function handleLogout() {
    await logout();
    await refresh();
    navigate('/login');
  }

  return (
    <div className="dashboard-page">
      <header className="dashboard-header">
        <h1>Product Twin</h1>
        <button type="button" onClick={handleLogout}>
          Logout
        </button>
      </header>
      <main>
        <p>Welcome to your digital twin dashboard.</p>
        <Link to="/upload">Upload File</Link>
        <GraphViewPlaceholder />
      </main>
    </div>
  );
}
