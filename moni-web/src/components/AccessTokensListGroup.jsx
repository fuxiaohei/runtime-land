import { ListGroup, Button } from "react-bootstrap";
import { TbWebhook, TbTrash, TbSquareKey } from "react-icons/tb";
import React from "react";

function AccessTokensListGroup({ tokens, onRemoveClick }) {
  console.log("--tokens", tokens, onRemoveClick);
  const listItems = tokens.map((token) => (
    <ListGroup.Item
      key={token.uuid}
      className="d-flex py-3 justify-content-between"
    >
      <div className="desc">
        {token.origin == "dashboard" ? (
          <TbSquareKey size={20} />
        ) : (
          <TbWebhook size={20} />
        )}
        <span className="ps-1 align-text-top fw-bold">{token.name}</span>
        <span className="ps-2 extra">
          Logged in 2 days ago, expires in 4 hours
        </span>
      </div>
      <Button
        onClick={() => {
          onRemoveClick(token);
        }}
        variant="link"
        size="sm"
        className="del-button"
      >
        <TbTrash size={20} />
      </Button>
    </ListGroup.Item>
  ));
  return (
    <ListGroup className="access-tokens-list" id="access-tokens-existing-list">
      {listItems}
    </ListGroup>
  );
}

export default AccessTokensListGroup;
