import { Col, Button, ListGroup, Spinner, Alert } from "react-bootstrap";
import { useEffect, useState } from "react";
import {
  createDeploymentToken,
  listDeploymentTokens,
  removeToken,
} from "../api/token";
import { RiDeleteBinLine } from "react-icons/ri";
import { DateTime } from "luxon";
import TokenCreateModal from "../components/TokenCreateModal";
import TokenRemoveModal from "../components/TokenRemoveModal";
import TokenNewCard from "../components/TokenNewCard";

function TokensContainer() {
  const [createModalShow, setCreateModalShow] = useState(false);
  const [createModalAlert, setCreateModalAlert] = useState("");
  const [removeModalShow, setRemoveModalShow] = useState(false);
  const [removeModalAlert, setRemoveModalAlert] = useState("");
  const [rmToken, setRmToken] = useState(null);

  const [newToken, setNewToken] = useState(null);
  const handleNewTokenDone = () => {
    setNewToken(null);
    fetchTokens();
  };

  const handleCreate = async (tokenDesc) => {
    let response = await createDeploymentToken(tokenDesc);
    if (!response.error) {
      setNewToken(response);
      setCreateModalShow(false);
    } else {
      setCreateModalAlert(response.error);
    }
    return response;
  };

  const handleRemove = async () => {
    let response = await removeToken(rmToken.uuid);
    if (response.error) {
      setRemoveModalAlert(response.error);
    } else {
      await fetchTokens();
      setRemoveModalShow(false);
    }
  };

  const renderNewToken = () => {
    if (!newToken) {
      return null;
    }
    return <TokenNewCard token={newToken} onDone={handleNewTokenDone} />;
  };

  const [isLoading, setIsLoading] = useState(true);
  const [tokens, setTokens] = useState([]);
  const [tokensListAlert, setTokensListAlert] = useState("");

  const fetchTokens = async () => {
    setIsLoading(true);
    let response = await listDeploymentTokens();
    if (response.error) {
      setIsLoading(false);
      setTokensListAlert(response.error);
      return;
    }
    setTokens(response);
    setIsLoading(false);
  };

  useEffect(() => {
    fetchTokens();
  }, []);

  const renderTokens = () => {
    if (isLoading) {
      return (
        <Spinner
          className="my-3"
          variant="secondary"
          animation="border"
          role="status"
        >
          <span className="visually-hidden">Loading...</span>
        </Spinner>
      );
    }

    if (tokensListAlert) {
      return <Alert variant="danger">Loading failure, {tokensListAlert}</Alert>;
    }

    return tokens.map((token) => {
      return (
        <ListGroup.Item
          key={token.uuid}
          className="d-flex justify-content-between lh-lg"
        >
          <div>
            <strong>{token.name}</strong>
            <span className="ms-2 text-secondary">
              expires in{" "}
              {DateTime.fromSeconds(token.expired_at)
                .setLocale("en-US")
                .toLocaleString(DateTime.DATE_FULL)}
            </span>
          </div>
          <div>
            <span
              onClick={() => {
                setRmToken(token);
                setRemoveModalShow(true);
              }}
            >
              <RiDeleteBinLine size={21} role="button" />
            </span>
          </div>
        </ListGroup.Item>
      );
    });
  };

  return (
    <Col md={8}>
      <div className="mt-3 d-flex justify-content-between">
        <h4>Deployment Tokens</h4>
        <Button
          variant="primary"
          size="sm"
          onClick={() => setCreateModalShow(true)}
        >
          Create New Token
        </Button>
      </div>
      <hr />
      <p className="text-secondary">
        Deployment tokens are used to authenticate with the api when deploying a
        project via the command line.
      </p>
      <pre className="text-secondary">
        land-cli deploy --token=&lt;token&gt;
      </pre>
      {renderNewToken()}
      <ListGroup className="tokens-list">{renderTokens()}</ListGroup>
      <TokenCreateModal
        show={createModalShow}
        onClose={() => {
          setCreateModalShow(false);
          setCreateModalAlert("");
        }}
        onCreate={handleCreate}
        alert={createModalAlert}
      />
      <TokenRemoveModal
        show={removeModalShow}
        onClose={() => {
          setRemoveModalShow(false);
          setRemoveModalAlert("");
        }}
        name={removeToken?.name || ""}
        onRemove={handleRemove}
        alert={removeModalAlert}
      />
    </Col>
  );
}

export default TokensContainer;
