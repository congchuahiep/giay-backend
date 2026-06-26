### 1. Xem danh sách thành viên (List Members)

- Method & Path: `GET /{workspace_slug}/members`
- Quyền hạn (Extractor): `WorkspaceMember` (Bất kỳ ai đã là thành viên đều có thể xem danh sách đồng nghiệp của mình).
- Mục đích: Trả về danh sách tất cả những người đang ở trong Workspace.
- Dữ liệu trả về (DTO): Cần join bảng workspace_membership với bảng user để lấy ra user_id , email , tên , avatar_url , role (Quyền hạn trong workspace), và joined_at (Ngày tham gia).

### 2. Cập nhật quyền của thành viên (Update Member Role)

- Method & Path: `PATCH /{workspace_slug}/members/{user_id}`
- Quyền hạn (Extractor): `WorkspaceModerator` hoặc `WorkspaceOwner` .
- Dữ liệu gửi lên: `{ "role": "moderator" }`
- Mục đích: Thăng chức hoặc giáng chức một thành viên.
- Logic cần lưu ý (Bảo mật):
    - Moderator không thể thăng cấp người khác lên thành Owner .
    - Moderator không thể giáng chức hoặc thay đổi quyền của Owner .
    - Nếu Owner muốn chuyển giao quyền sở hữu cho người khác, sẽ có logic đặc biệt (hoặc tự động đổi Owner cũ thành Moderator).

### 3. Đuổi thành viên (Remove / Kick Member)

- Method & Path: `DELETE /{workspace_slug}/members/{user_id}`
- Quyền hạn (Extractor): `WorkspaceModerator` hoặc `WorkspaceOwner` .
- Mục đích: Xóa một user khỏi workspace. User đó sẽ bị mất quyền truy cập hoàn toàn.
- Logic cần lưu ý: Moderator không thể kick Owner .

### 4. Tự rời khỏi Workspace (Leave Workspace)

- Method & Path: `DELETE /{workspace_slug}/members/me` (hoặc `POST /{workspace_slug}/leave` )
- Quyền hạn (Extractor): `WorkspaceMember`
- Mục đích: Dành cho trường hợp người dùng chán và muốn tự rút lui khỏi Workspace.
- Logic cần lưu ý: Nếu người đang gọi API là Owner DUY NHẤT của Workspace, hệ thống phải chặn lại và báo lỗi: "Bạn là chủ sở hữu duy nhất, hãy chuyển giao quyền Owner cho người khác trước khi rời đi, hoặc hãy xóa hoàn toàn Workspace."
