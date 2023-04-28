import { Container, Card, ListGroup, Button } from "react-bootstrap";
import DashboardNavbar from "../components/DashboardNavbar";
import { TbWebhook, TbTrash, TbSquareKey } from "react-icons/tb";

function SettingsPage() {
  return (
    <div>
      <DashboardNavbar />
      <Container id="account-settings-container">
        <header id="account-settings-header">
          <h2>Account Settings</h2>
        </header>
        <Card id="access-tokens-container" className="account-settings-card">
          <Card.Body>
            <Card.Title id="access-tokens">Access Tokens</Card.Title>
            <Card.Subtitle>
              Personal access tokens can be used to access Moni-Web API.
            </Card.Subtitle>
            <ListGroup id="access-tokens-list">
              <ListGroup.Item className="d-flex py-3 justify-content-between">
                <div className="desc">
                  <TbWebhook size={20} />
                  <span className="ps-1 align-text-top fw-bold">
                    Web Page Login
                  </span>
                  <span className="ps-2 extra">
                    Logged in 2 days ago, expires in 4 hours
                  </span>
                </div>
                <Button variant="link" size="sm" className="del-button">
                  <TbTrash size={20} />
                </Button>
              </ListGroup.Item>
              <ListGroup.Item className="d-flex py-3 justify-content-between">
                <div className="desc">
                  <TbSquareKey size={20} />
                  <span className="ps-1 align-text-top fw-bold">
                    User Created
                  </span>
                  <span className="ps-2 extra">
                    Logged in 2 days ago, expires in 4 hours
                  </span>
                </div>
                <Button variant="link" className="del-button">
                  <TbTrash size={20} />
                </Button>
              </ListGroup.Item>
            </ListGroup>
            <Card.Text>
              <Button variant="dark" size="sm">
                Create Access Token
              </Button>
            </Card.Text>
          </Card.Body>
        </Card>
      </Container>
    </div>
  );
}

export default SettingsPage;
