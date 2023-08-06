import { Container, Row, Col, Card, Button } from "react-bootstrap";
import { ProjectLayout } from "../layouts/Layout";
import { LiaExternalLinkAltSolid } from "react-icons/lia";
import { useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { get_overview } from "../api/projects";
import {
  ProjectNoProductionCard,
  ProjectProdutionCard,
} from "../components/ProjectProductionCard";
import ProjectDeploymentsTable from "../components/ProjectDeploymentsTable";

function ProjectStatusButton({ status, url }) {
  if (status == "pending") {
    return (
      <Button
        disabled
        size="sm"
        variant="secondary"
        className="align-text-bottom"
      >
        Pending
      </Button>
    );
  }
  if (url) {
    return (
      <Button
        size="sm"
        variant="success"
        href={url}
        className="align-text-bottom"
      >
        <LiaExternalLinkAltSolid className="me-1" />
        View
      </Button>
    );
  }
  return null;
}

function ProjectOverviewPage() {
  let { projectName } = useParams();
  let queryKey = "project-overview|" + projectName;

  const {
    isLoading,
    isError,
    error,
    data: overview,
  } = useQuery({
    queryKey: [queryKey],
    queryFn: async () => {
      const data = await get_overview(projectName);
      return data;
    },
    retry: false,
  });

  console.log("---", overview);
  let project = overview?.project || {};

  return (
    <ProjectLayout
      title={projectName + " | Projects | Runtime.land"}
      projectName={projectName}
    >
      <div id="overview-header" className="mb-5 mt-3">
        <h3 className="d-inline">{projectName}</h3>
        <span className="prod-link text-secondary ps-3">
          <ProjectStatusButton status={project.status} url={project.prod_url} />
        </span>
      </div>
      <Container fluid className="p-0">
        <Row>
          <Col lg={6} xl={4}>
            {overview?.prod_deployment ? (
              <ProjectProdutionCard
                deployment={overview?.prod_deployment || {}}
              />
            ) : (
              <ProjectNoProductionCard />
            )}
          </Col>
          <Col lg={6} xl={8}>
            <Card>
              <Card.Header>
                Deployments ({overview?.deployments.length || 0})
              </Card.Header>
              <Card.Body>
                <Card.Text as="div">
                  <p className="text-secondary">
                    All deployments of this project.
                  </p>
                </Card.Text>
                <ProjectDeploymentsTable
                  deployments={overview?.deployments || []}
                />
              </Card.Body>
            </Card>
          </Col>
        </Row>
      </Container>
    </ProjectLayout>
  );
}

export default ProjectOverviewPage;
