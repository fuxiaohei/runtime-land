import { useState } from "react";
import { Button } from "react-bootstrap";
import { CopyToClipboard } from "react-copy-to-clipboard";
import { FaCheck, FaRegCopy } from "react-icons/fa";

function TokenNewCard({ token, onDone }) {
  const newToken = token;
  const [copied, setCopied] = useState(false);
  return (
    <div className="new-token border p-3 rounded d-flex justify-content-between mb-4">
      <div>
        <p className="mb-1">
          <strong>{newToken.name}</strong>
        </p>
        <p className="mb-1">
          {newToken.value}
          <CopyToClipboard
            role="button"
            text={newToken.value}
            onCopy={() => setCopied(true)}
          >
            {copied ? (
              <FaCheck className="ms-2" />
            ) : (
              <FaRegCopy className="ms-2" />
            )}
          </CopyToClipboard>
        </p>
      </div>
      <Button variant="success" onClick={onDone}>
        Done
      </Button>
    </div>
  );
}

export default TokenNewCard;
