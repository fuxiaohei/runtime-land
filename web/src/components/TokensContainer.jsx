import { Col, Button, ListGroup, Spinner, Alert } from "react-bootstrap";
import { useState } from "react";
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
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

function TokensContainer() {
  const queryClient = useQueryClient();

  const [createModalShow, setCreateModalShow] = useState(false);
  const [createModalAlert, setCreateModalAlert] = useState("");
  const [removeModalShow, setRemoveModalShow] = useState(false);
  const [removeModalAlert, setRemoveModalAlert] = useState("");
  const [rmToken, setRmToken] = useState(null);

  const [newToken, setNewToken] = useState(null);
  const handleNewTokenDone = () => {
    setNewToken(null);
    queryClient.invalidateQueries({ queryKey: ["tokens"] });
  };

  const createMutation = useMutation({
    mutationFn: async (tokenDesc) => {
      return await createDeploymentToken(tokenDesc);
    },
    onSuccess: async (data) => {
      setNewToken(data);
      setCreateModalShow(false);
    },
    onError: (error) => {
      setCreateModalAlert(error.toString());
    },
  });

  const removeMutation = useMutation({
    mutationFn: async () => {
      await removeToken(rmToken.uuid);
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["tokens"] });
      setRemoveModalShow(false);
    },
    onError: (error) => {
      setRemoveModalAlert(error.toString());
    },
  });

  const renderNewToken = () => {
    if (!newToken) {
      return null;
    }
    return <TokenNewCard token={newToken} onDone={handleNewTokenDone} />;
  };

  const {
    isLoading,
    isError,
    error,
    data: tokens,
  } = useQuery({
    queryKey: ["tokens"],
    queryFn: listDeploymentTokens,
    retry: false,
  });

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

    if (isError) {
      return (
        <Alert variant="danger">Loading failure, {error.toString()}</Alert>
      );
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
        onCreate={(tokenDesc) => createMutation.mutate(tokenDesc)}
        alert={createModalAlert}
      />
      <TokenRemoveModal
        show={removeModalShow}
        onClose={() => {
          setRemoveModalShow(false);
          setRemoveModalAlert("");
        }}
        name={removeToken?.name || ""}
        onRemove={() => removeMutation.mutate()}
        alert={removeModalAlert}
      />
    </Col>
  );
}

export default TokensContainer;
