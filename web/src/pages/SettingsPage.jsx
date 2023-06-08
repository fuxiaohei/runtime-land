import { Container, Card, Button, Spinner } from "react-bootstrap";
import DashboardNavbar from "../components/DashboardNavbar";
import AccessTokenCreateModal from "../components/AccessTokenCreateModal";
import AccessTokenCreatedItem from "../components/AccessTokenCreatedItem";
import AccessTokensListGroup from "../components/AccessTokensListGroup";
import React, { useEffect } from "react";
import {
  createAccessToken,
  listAccessTokens,
  removeAccessToken,
} from "../api/token";
import AccessTokenRemoveModal from "../components/AccessTokenRemoveModal";

function SettingsPage() {
  const [tokenModelShow, setTokenModelShow] = React.useState(false);
  const [removeModelShow, setRemoveModelShow] = React.useState({
    show: false,
    token: null,
  });
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
    fetchTokens();
  };

  const handleRemoveClick = async (token) => {
    setRemoveModelShow({ show: true, token: token });
  };

  const fetchTokens = async () => {
    let response = await listAccessTokens();
    if (response.error) {
      return;
    }
    setTokensList(response.dataList || []);
  };

  const handleRemoveSubmit = async (token) => {
    let response = await removeAccessToken(token.uuid);
    if (response.error) {
      return;
    }
    setRemoveModelShow({ show: false, token: null });
    fetchTokens();
  };

  useEffect(() => {
    if (tokensList.length) {
      return;
    }

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
              <AccessTokensListGroup
                tokens={tokensList}
                onRemoveClick={handleRemoveClick}
              />
            ) : (
              <Spinner
                className="access-tokens-loading"
                animation="border"
                size="sm"
              />
            )}
            {createdToken ? (
              <AccessTokenCreatedItem
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
      <AccessTokenCreateModal
        show={tokenModelShow}
        onHide={() => setTokenModelShow(false)}
        onSubmit={handleCreateSubmit}
      />
      <AccessTokenRemoveModal
        show={removeModelShow.show}
        onHide={() => setRemoveModelShow({ show: false, token: null })}
        token={removeModelShow.token}
        onSubmit={handleRemoveSubmit}
      />
    </div>
  );
}

export default SettingsPage;
