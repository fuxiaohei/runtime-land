import { useQuery } from "@tanstack/react-query";
import { Badge, Card, ListGroup } from "react-bootstrap";
import { listRegions } from "../api/regions";
import LoadingPage from "../pages/Loading";

function RegionsPanel() {
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

  if (isLoading) {
    return <LoadingPage />;
  }

  const renderRow = (region) => {
    let status_bg = region.status == "active" ? "success" : "warning";
    return (
      <ListGroup.Item key={region.key}>
        <span>{region.name}</span>
        <Badge bg={status_bg} className="ms-2">
          {region.status}
        </Badge>
        <Badge bg="secondary" className="ms-2">
          {region.runtimes}
        </Badge>
      </ListGroup.Item>
    );
  };

  return (
    <Card>
      <Card.Header className="bg-primary-subtle">Regions</Card.Header>
      <Card.Body>
        <p>The following regions are available for deployment:</p>
        <ListGroup variant="flush">
          {regions.map((region) => renderRow(region))}
        </ListGroup>
      </Card.Body>
    </Card>
  );
}

export default RegionsPanel;
