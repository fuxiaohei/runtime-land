import { Container, Card, Button, Spinner } from "react-bootstrap";
import DashboardNavbar from "../components/DashboardNavbar";
import CreateAccessTokenModal from "../components/CreateAccessTokenModal";
import CreatedTokenListGroup from "../components/CreatedTokenListGroup";
import ListAccessTokensGroup from "../components/ListAccessTokensGroup";
import React, { useEffect } from "react";
import { createAccessToken, listAccessTokens } from "../api/token";

function SettingsPage() {
  const [tokenModelShow, setTokenModelShow] = React.useState(false);
  const [createdToken, setCreatedToken] = React.useState(null);
  const [tokensList, setTokensList] = React.useState([]);

  const handleCreateSubmit = async (event) => {
    const form = event.currentTarget;
    const formData = new FormData(form);
    const tokenName = formData.get("tokenvalue");

    event.preventDefault();
    event.stopPropagation();

    let response = await createAccessToken(tokenName);
    if (response.error) {
      return;
    }
    setCreatedToken(response.data);
    setTokenModelShow(false);
  };

  const handleDoneClick = async () => {
    // TODO: load the list of tokens again
    setCreatedToken(null);
  };

  useEffect(() => {
    const fetchTokens = async () => {
      let response = await listAccessTokens();
      if (response.error) {
        return;
      }
      setTokensList(response.dataList || []);
    };
    fetchTokens();
  });

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
            {tokensList.length ? (
              <ListAccessTokensGroup tokens={tokensList} />
            ) : (
              <Spinner
                className="access-tokens-loading"
                animation="border"
                size="sm"
              />
            )}
            {createdToken ? (
              <CreatedTokenListGroup
                onDoneClick={handleDoneClick}
                value={createdToken}
              />
            ) : null}
            <Card.Text>
              <Button
                variant="dark"
                size="sm"
                onClick={() => setTokenModelShow(true)}
              >
                Create Access Token
              </Button>
            </Card.Text>
          </Card.Body>
        </Card>
      </Container>
      <CreateAccessTokenModal
        show={tokenModelShow}
        onHide={() => setTokenModelShow(false)}
        onSubmit={handleCreateSubmit}
      />
    </div>
  );
}

export default SettingsPage;
