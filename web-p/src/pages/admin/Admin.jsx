import { useQuery } from "@tanstack/react-query";
import AdminLayout from "../../layouts/Admin";
import { list_regions } from "../../api/regions";
import {
  Col,
  Container,
  Row,
  Table,
  Badge,
  Card,
  Alert,
} from "react-bootstrap";

function RegionsTable({ regions, loading, error }) {
  if (loading) {
    return <Alert variant="info">Loading...</Alert>;
  }
  if (error) {
    return <Alert variant="danger">{error.toString()}</Alert>;
  }

  return (
    <div className="regions-table-container">
      <Table id="regions-table" className="mb-0" hover>
        <thead>
          <tr>
            <th>Name</th>
            <th style={{ width: "100px" }}>Runtimes</th>
            <th style={{ width: "100px" }}>Status</th>
          </tr>
        </thead>
        <tbody>
          {regions.map((region) => (
            <tr key={region.id}>
              <td>{region.key}</td>
              <td>{region.runtimes}</td>
              <td>
                <Badge
                  bg={region.status == "active" ? "success" : "warning"}
                >
                  {region.status}
                </Badge>
              </td>
            </tr>
          ))}
        </tbody>
      </Table>
    </div>
  );
}

function AdminPage() {
  const {
    isLoading,
    error,
    data: regions,
  } = useQuery({
    queryKey: ["regions-list"],
    queryFn: list_regions,
    retry: false,
  });
  return (
    <AdminLayout title="Admin | Runtime.land">
      <Container fluid id="admin-container" className="p-0">
        <Row>
          <Col lg={6}>
            <Card>
              <Card.Header>Regions</Card.Header>
              <Card.Body>
                <RegionsTable
                  regions={regions}
                  error={error?.toString()}
                  loading={isLoading}
                />
              </Card.Body>
            </Card>
          </Col>
          <Col lg={6}></Col>
        </Row>
      </Container>
    </AdminLayout>
  );
}

export default AdminPage;
