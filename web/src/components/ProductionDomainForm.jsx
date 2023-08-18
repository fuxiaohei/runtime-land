import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { Button, Card, Col, Form, Row } from "react-bootstrap";
import { listDomainSettings } from "../api/regions";
import LoadingPage from "../pages/Loading";

function ProductionDomainForm() {
  const [domainSuffix, setDomainSuffix] = useState("");
  const [domainProtocol, setDomainProtocol] = useState("");

  const {
    isLoading,
    isError,
    error,
    data: settings,
  } = useQuery({
    queryKey: ["settings-domain"],
    queryFn: listDomainSettings,
    retry: false,
  });

  if (isLoading) {
    return <LoadingPage />;
  }

  if (settings && !domainSuffix && !domainProtocol) {
    setDomainProtocol(settings["production_protocol"]);
    setDomainSuffix(settings["production_domain"]);
  }

  const handleSubmit = (e) => {
    e.preventDefault();
  };

  return (
    <Card>
      <Card.Header>Production Domain</Card.Header>
      <Card.Body>
        <p>
          The domain suffix and protocol for production deployments:
          [protocol]://your-domain.[suffix]
        </p>
        <Form onSubmit={handleSubmit}>
          <Form.Group as={Row} className="mb-3">
            <Form.Label column sm={4}>
              Domain Suffix
            </Form.Label>
            <Col sm={8}>
              <Form.Control
                defaultValue={domainSuffix}
                type="text"
                placeholder="domain suffix"
                onChange={(e) => setDomainSuffix(e.target.value)}
                required
              />
            </Col>
          </Form.Group>
          <Form.Group as={Row} className="mb-3">
            <Form.Label column sm={4}>
              Protocol
            </Form.Label>
            <Col sm={8}>
              <Form.Select
                name="protocol"
                defaultValue={domainProtocol}
                onChange={(e) => setDomainProtocol(e.target.value)}
              >
                <option value="https">https</option>
                <option value="http">http</option>
              </Form.Select>
            </Col>
          </Form.Group>
          <Form.Group as={Row} className="mb-3">
            <Col sm={{ span: 8, offset: 4 }}>
              <Button type="submit">Save</Button>
            </Col>
          </Form.Group>
        </Form>
      </Card.Body>
    </Card>
  );
}

export default ProductionDomainForm;
