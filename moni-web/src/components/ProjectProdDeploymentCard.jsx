import { Card, ListGroup } from "react-bootstrap";
import { ButtonLink } from "./ButtonLink";
import { BsClock } from "react-icons/bs";

function ProjectProdDeploymentCard() {
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
              <a href="https://quick-trout-91.deno.dev/" target="_blank">
                quick-trout-91.deno.dev
              </a>
            </ListGroup.Item>
            <ListGroup.Item>
              <a
                href="https://quick-trout-91-m9t91f4cbqh0.deno.dev/"
                target="_blank"
              >
                quick-trout-91-m9t91f4cbqh0.deno.dev
              </a>
            </ListGroup.Item>
            <ListGroup.Item className="text-end small text-muted text-uppercase">
              <BsClock size={10} className="me-1"/>3 months ago
            </ListGroup.Item>
          </ListGroup>
        </div>
      </Card.Body>
    </Card>
  );
}

export default ProjectProdDeploymentCard;
