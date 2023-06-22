import { ListGroup, Button } from "react-bootstrap";
import { TbWebhook, TbTrash, TbSquareKey } from "react-icons/tb";
import React from "react";
import TimeAgo from "javascript-time-ago";

function AccessTokensListGroup({ tokens, onRemoveClick }) {
  const timeAgo = new TimeAgo("en-US");
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
          Logged {timeAgo.format(token.updated_at * 1000)}, expires{" "}
          {timeAgo.format(token.expired_at * 1000)}
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
