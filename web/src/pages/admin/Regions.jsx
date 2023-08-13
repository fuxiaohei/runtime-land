import { Badge, Container, ListGroup } from "react-bootstrap";
import { AuthProvider } from "../../layouts/AuthContext";
import MainLayout from "../../layouts/MainLayout";
import AdminNavHeader from "../../components/AdminNavHeader";
import { useQuery } from "@tanstack/react-query";
import { listRegions } from "../../api/regions";
import QueryWrapper from "../../layouts/QueryWrapper";

function AdminRegionsPage() {
  const {
    isLoading,
    isError,
    error,
    data: regions,
  } = useQuery({
    queryKey: ["regions-list"],
    queryFn: listRegions,
    retry: false,
  });

  const renderRow = (region) => {
    let status_bg = region.status == "active" ? "success" : "warning";
    return (
      <ListGroup.Item key={region.key}>
        <span className="fw-bold">{region.name}</span>
        <span className="text-secondary ms-3">({region.key})</span>
        <Badge bg={status_bg} className="ms-3">
          {region.status}
        </Badge>
        <Badge bg="secondary" className="ms-3">
          {region.runtimes}
        </Badge>
      </ListGroup.Item>
    );
  };

  return (
    <AuthProvider>
      <MainLayout title="Regions | Admin Panel | Runtime.land">
        <Container id="admin-page" className="mt-4">
          <h3 className="mb-3">Admin Panel</h3>
          <AdminNavHeader activeKey="regions" />
          <p className="text-secondary py-2">
            The following regions are registered with Runtime.land:
          </p>
          <ListGroup className="lh-lg" variant="flush">
            <QueryWrapper isLoading={isLoading} isError={isError} error={error}>
              {(regions || []).map((region) => renderRow(region))}
            </QueryWrapper>
          </ListGroup>
        </Container>
      </MainLayout>
    </AuthProvider>
  );
}

export default AdminRegionsPage;
