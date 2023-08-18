import { useMutation } from "@tanstack/react-query";
import { useState } from "react";
import { Button, Form, InputGroup, ListGroup } from "react-bootstrap";
import { BiTrash } from "react-icons/bi";
import ReactTimeAgo from "react-time-ago";
import { removeToken } from "../api/token";
import TokenNewCard from "./TokenNewCard";
import TokenRemoveModal from "./TokenRemoveModal";

function TokensList({
  tokens,
  newToken,
  handleNewToken,
  handleNewTokenDone,
  handleRemoveToken,
}) {
  const removeTokenMutation = useMutation({
    mutationFn: async (uuid) => {
      await removeToken(uuid);
    },
    onSuccess: async () => {
      setRemoveModalShow(false);
      setRemoveAlert("");
      await handleRemoveToken();
    },
    onError: (error) => {
      setRemoveAlert(error.toString());
    },
  });

  const renderRow = (token) => {
    return (
      <ListGroup.Item key={token.uuid} className="py-2">
        <div className="d-flex justify-content-between">
          <div className="lh-lg">
            <span className="token-name fw-bold pe-2 border-end d-inline-block">
              {token.name}
            </span>
            <span className="text-secondary border-end px-2">
              Active{" "}
              <ReactTimeAgo date={token.updated_at * 1000} locale="en-US" />
            </span>
            <span className="text-secondary px-2">
              Expired{" "}
              <ReactTimeAgo date={token.expired_at * 1000} locale="en-US" />
            </span>
          </div>
          <div>
            <Button
              className="trash-btn"
              variant="link"
              onClick={() => {
                setRemoveModalShow(true);
                setRmToken(token);
                setRemoveAlert("");
              }}
            >
              <BiTrash size={21} />
            </Button>
          </div>
        </div>
      </ListGroup.Item>
    );
  };

  const [validated, setValidated] = useState(false);
  const [tokenName, setTokenName] = useState("");

  const handleSubmit = async (event) => {
    const form = event.currentTarget;
    setValidated(true);

    if (form.checkValidity() === false) {
      event.preventDefault();
      event.stopPropagation();
      return;
    }

    event.preventDefault();
    await handleNewToken(tokenName);
    setTokenName("");
    form.reset();
    setValidated(false);
  };

  const [rmToken, setRmToken] = useState(null);
  const [removeModalShow, setRemoveModalShow] = useState(false);
  const [removeAlert, setRemoveAlert] = useState("");

  return (
    <div className="py-2" id="deployment-tokens-list">
      <div className="d-flex justify-content-between">
        <h5 className="mb-2 fw-bold align-middle lh-lg">Deployment Tokens</h5>
        <Form
          className="d-inline-block align-middle me-3"
          noValidate
          validated={validated}
          onSubmit={handleSubmit}
        >
          <InputGroup className="mb-3">
            <Form.Control
              placeholder="Enter token name"
              required
              onChange={(event) => setTokenName(event.target.value)}
            />
            <Button variant="outline-primary" type="submit">
              + New token
            </Button>
          </InputGroup>
        </Form>
      </div>
      {newToken ? (
        <TokenNewCard token={newToken} onDone={handleNewTokenDone} />
      ) : null}
      <ListGroup variant="flush">
        {tokens.map((token) => renderRow(token))}
      </ListGroup>
      <TokenRemoveModal
        show={removeModalShow}
        handleClose={() => setRemoveModalShow(false)}
        alert={removeAlert}
        handleRemove={() => {
          removeTokenMutation.mutate(rmToken.uuid);
        }}
      />
    </div>
  );
}

export default TokensList;
