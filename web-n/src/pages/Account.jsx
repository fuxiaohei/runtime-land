import { Container, Button } from "react-bootstrap";
import { BiLogoGithub } from "react-icons/bi";
import TokensList from "../components/TokensList";
import { AuthProvider } from "../layouts/AuthContext";
import MainLayout from "../layouts/MainLayout";

function AccountPage() {
  return (
    <AuthProvider>
      <MainLayout>
        <Container>
          <div className="account-info mt-5 px-5 py-4">
            <div className="d-flex pb-4 justify-content-between border-bottom">
              <div className="d-flex justify-content-start">
                <img
                  src="https://avatars.githubusercontent.com/u/2142787?v=4"
                  width="70"
                  height="70"
                  className="rounded-3"
                />
                <div className="info ms-4">
                  <h2 className="fs-3 fw-bold">FuXiaohei</h2>
                  <p className="email text-secondary">
                    <BiLogoGithub className="me-2" size={24} />
                    fuxiaohei@vip.qq.com
                  </p>
                </div>
              </div>
              <div>
                <Button variant="outline-success">Hobby Plan</Button>
              </div>
            </div>
          </div>
          <TokensList />
        </Container>
      </MainLayout>
    </AuthProvider>
  );
}

export default AccountPage;
