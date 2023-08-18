import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Alert, Button, Container, Form } from "react-bootstrap";
import { listDomainSettings, updateDomainSettings } from "../../api/regions";
import AdminNavHeader from "../../components/AdminNavHeader";
import { AuthProvider } from "../../layouts/AuthContext";
import MainLayout from "../../layouts/MainLayout";
import QueryWrapper from "../../layouts/QueryWrapper";

function AdminDomainsForm({ settings, onSubmit, isSuccess }) {
  const [domainSuffix, setDomainSuffix] = useState("");
  const [domainProtocol, setDomainProtocol] = useState("");

  if (settings && !domainSuffix && !domainProtocol) {
    setDomainProtocol(settings["production_protocol"]);
    setDomainSuffix(settings["production_domain"]);
  }

  const handleSubmit = (e) => {
    e.preventDefault();
    onSubmit({
      domain: domainSuffix,
      protocol: domainProtocol,
    });
  };

  return (
    <Form className="border-top pt-4" onSubmit={handleSubmit}>
      {isSuccess ? (
        <Alert className="mb-3" variant="success" dismissible>
          Domain settings updated successfully
        </Alert>
      ) : null}
      <Form.Group className="mb-3">
        <Form.Label>Domain Suffx</Form.Label>
        <Form.Control
          type="text"
          placeholder="enter domain suffix"
          defaultValue={domainSuffix}
          onChange={(e) => setDomainSuffix(e.target.value)}
          required
        />
        <Form.Text className="text-muted">
          Enter the domain suffix for production deployments
        </Form.Text>
      </Form.Group>
      <Form.Group className="mb-3">
        <Form.Label>Protocol</Form.Label>
        <Form.Select
          name="protocol"
          defaultValue={domainProtocol}
          onChange={(e) => setDomainProtocol(e.target.value)}
        >
          <option value="https">HTTPS</option>
          <option value="http">HTTP</option>
        </Form.Select>
        <Form.Text className="text-muted">
          Select the protocol for handling http requests
        </Form.Text>
      </Form.Group>
      <div className="text-start">
        <Button variant="primary" type="submit">
          Submit
        </Button>
      </div>
    </Form>
  );
}

function AdminDomainsPage() {
  const queryClient = useQueryClient();
  const [isSuccess, setSuccess] = useState(false);
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

  const domainsMutation = useMutation({
    mutationFn: updateDomainSettings,
    onSuccess: () => {
      queryClient.invalidateQueries("settings-domain");
      setSuccess(true);
    },
    onError: (error) => {},
  });

  const handleSubmit = ({ domain, protocol }) => {
    setSuccess(false);
    domainsMutation.mutate({
      domain: domain,
      protocol: protocol,
    });
  };

  return (
    <AuthProvider>
      <MainLayout title="Domains | Admin Panel | Runtime.land">
        <Container id="admin-page" className="mt-4">
          <h3 className="mb-3">Admin Panel</h3>
          <AdminNavHeader activeKey="domains" />
          <p className="text-secondary py-2">
            Domains setting is used to configure the domains that Runtime.land
            deploys to. Format as follows: <code>[protocol]</code>://project.
            <code>[domain]</code>/
          </p>
          <QueryWrapper isLoading={isLoading} isError={isError} error={error}>
            <AdminDomainsForm
              isSuccess={isSuccess}
              settings={settings}
              onSubmit={handleSubmit}
            />
          </QueryWrapper>
        </Container>
      </MainLayout>
    </AuthProvider>
  );
}

export default AdminDomainsPage;
