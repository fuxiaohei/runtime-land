import { Container, Row } from "react-bootstrap";
import { DefaultLayout } from "../layouts/Layout";
import AccountInfoContainer from "../components/AccountInfoContainer";
import TokensContainer from "../components/TokensContainer";

function AccountPage() {
  return (
    <DefaultLayout title="Account | Runtime.land">
      <Container fluid id="account-container">
        <Row>
          <AccountInfoContainer />
          <TokensContainer />
        </Row>
      </Container>
    </DefaultLayout>
  );
}

export default AccountPage;
