import { ListGroup, Button } from "react-bootstrap";
import { TbWebhook, TbTrash, TbSquareKey } from "react-icons/tb";
import React from "react";

function ListAccessTokensGroup({ tokens }) {
  return (
    <ListGroup className="access-tokens-list" id="access-tokens-existing-list">
      <ListGroup.Item className="d-flex py-3 justify-content-between">
        <div className="desc">
          <TbWebhook size={20} />
          <span className="ps-1 align-text-top fw-bold">Web Page Login</span>
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
          <span className="ps-1 align-text-top fw-bold">User Created</span>
          <span className="ps-2 extra">
            Logged in 2 days ago, expires in 4 hours
          </span>
        </div>
        <Button variant="link" className="del-button">
          <TbTrash size={20} />
        </Button>
      </ListGroup.Item>
    </ListGroup>
  );
}

export default ListAccessTokensGroup;
