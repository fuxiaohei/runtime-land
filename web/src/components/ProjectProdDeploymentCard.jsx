import { Card, ListGroup } from "react-bootstrap";
import { ButtonLink } from "./ButtonLink";
import { BsClock } from "react-icons/bs";
import TimeAgo from "javascript-time-ago";

function ProjectProdDeploymentCard({ deployment }) {
  const timeAgo = new TimeAgo("en-US");
  return (
    <Card className="project-prod-card mb-3">
      <Card.Body>
        <Card.Title className="d-flex justify-content-between">
          <div>Production Deployment</div>
          <div className="deployment-prod-logs">
            <ButtonLink to="./logs" variant="success" disabled>
              Logs
            </ButtonLink>
          </div>
        </Card.Title>
        <div>
          <ListGroup variant="flush" className="project-prod-domains">
            <ListGroup.Item>
              <a href={deployment.prod_url} target="_blank">
                {new URL(deployment.prod_url).host}
              </a>
            </ListGroup.Item>
            <ListGroup.Item>
              <a href={deployment.domain_url} target="_blank">
                {new URL(deployment.domain_url).host}
              </a>
            </ListGroup.Item>
            <ListGroup.Item className="text-end small text-muted text-uppercase">
              <BsClock size={10} className="me-1" />
              {timeAgo.format(deployment.updated_at * 1000)}
            </ListGroup.Item>
          </ListGroup>
        </div>
      </Card.Body>
    </Card>
  );
}

export default ProjectProdDeploymentCard;
