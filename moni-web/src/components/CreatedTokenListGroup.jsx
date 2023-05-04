import React from "react";
import { CopyToClipboard } from "react-copy-to-clipboard";
import { ListGroup, Button } from "react-bootstrap";

function CreatedTokenListGroup({ value, onDoneClick }) {
  const [tokenCopied, setTokenCopied] = React.useState(false);
  return (
    <ListGroup className="access-tokens-list" id="access-tokens-new-value">
      <ListGroup.Item className="d-flex py-3 justify-content-between">
        <div className="desc">
          <p className="name">
            <TbSquareKey size={20} />
            <span className="ps-1 align-text-top fw-bold">{value.name}</span>
          </p>
          <p className="value">
            {value.value}
            <CopyToClipboard
              onCopy={() => {
                setTokenCopied(true);
              }}
              text={value.value}
              className="copy-btn"
            >
              {tokenCopied ? <TbCheck size={20} /> : <TbCopy size={20} />}
            </CopyToClipboard>
          </p>
        </div>
        <div className="new-btn">
          <Button onClick={onDoneClick} variant="success" size="sm">
            Done
          </Button>
        </div>
      </ListGroup.Item>
    </ListGroup>
  );
}

export default CreatedTokenListGroup;
