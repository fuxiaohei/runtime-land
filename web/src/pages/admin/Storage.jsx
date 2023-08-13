import { Container } from "react-bootstrap";
import { AuthProvider } from "../../layouts/AuthContext";
import MainLayout from "../../layouts/MainLayout";
import AdminNavHeader from "../../components/AdminNavHeader";

function AdminStoragePage() {
  return (
    <AuthProvider>
      <MainLayout title="Storage | Admin Panel | Runtime.land">
        <Container id="admin-page" className="mt-4">
          <h3 className="mb-3">Admin Panel</h3>
          <AdminNavHeader activeKey="storage" />
          <p>Storage</p>
        </Container>
      </MainLayout>
    </AuthProvider>
  );
}

export default AdminStoragePage;
