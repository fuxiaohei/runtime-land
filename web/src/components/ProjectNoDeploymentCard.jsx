import { Card } from "react-bootstrap";

function ProjectNoDeploymentCard() {
  return (
    <Card className="project-no-prod-card mb-3">
      <Card.Body>
        <Card.Title>No Production Deployment</Card.Title>
        <p className="text-muted">
          Publish a preview deployment to Production to get started, <br />
          or use <span>land-cli deploy --production</span> to deploy directly
          from your local project.
        </p>
      </Card.Body>
    </Card>
  );
}

export default ProjectNoDeploymentCard;
