import { Col, Row, Container } from "react-bootstrap";
import { AuthProvider } from "../../layouts/AuthContext";
import MainLayout from "../../layouts/MainLayout";
import RegionsPanel from "../../components/RegionsPanel";
import ProductionDomainForm from "../../components/ProductionDomainForm";
import AdminNavHeader from "../../components/AdminNavHeader";

function AdminPage() {
  return (
    <AuthProvider>
      <MainLayout title="Admin Panel | Runtime.land">
        <Container id="admin-page" className="mt-4">
          <h3 className="mb-3">Admin Panel</h3>
          <AdminNavHeader />
          <Row>
            <Col>
              <RegionsPanel />
            </Col>
            <Col>
              <ProductionDomainForm />
            </Col>
          </Row>
        </Container>
      </MainLayout>
    </AuthProvider>
  );
}

export default AdminPage;
