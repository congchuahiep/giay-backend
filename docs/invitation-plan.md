1. **Tạo lời mời (Create Invitation):**
    - Người gửi (yêu cầu quyền Moderator hoặc Owner) gửi lời mời tới một địa chỉ email với một role cụ thể.
    - Kiểm tra xem email đó đã là thành viên của workspace chưa.
    - Kiểm tra xem email đó đã có lời mời nào đang chờ (active) chưa. (Nếu có lời mời cũ nhưng `is_used=true` do từng là thành viên rồi bị kick, hệ thống sẽ xóa lời mời cũ đó đi và tạo mới).
    - Tạo bản ghi lời mời và gọi Task gửi Email chứa link đính kèm token .
2. **Gửi lại lời mời (Resend Invitation):**
    - Hủy bỏ bản ghi lời mời cũ, tạo một bản ghi lời mời mới với `expires_at` mới và tiến hành gửi lại Email.
3. **Thu hồi lời mời (Revoke Invitation):**
    - Cập nhật trường `revoked_at` bằng thời gian hiện tại. Lời mời sẽ lập tức mất hiệu lực.
4. **Xem thông tin lời mời (Preview Invitation):**
    - Khi người được mời nhấp vào link từ email, một API Public sẽ lấy ra thông tin tóm tắt dựa trên token (tên workspace, role được mời) để hiển thị trên giao diện xác nhận.
5. **Chấp nhận lời mời (Accept Invitation):**
    - Yêu cầu người dùng phải đăng nhập tài khoản.
    - Kiểm tra tính hợp lệ của token: chưa hết hạn, chưa bị thu hồi, chưa được sử dụng.
    - Xác thực Email: Kiểm tra email của tài khoản đang đăng nhập có khớp với email của lời mời hay không (ngăn chặn việc lấy link của người khác để join).
    - Sử dụng Transaction để cập nhật is_used = true và chèn một bản ghi mới vào bảng workspace_membership .

---

Xử lý workspace, một workspace đảm bảo chỉ có 1 owner duy nhất, và cũng như là khi mời thành viên vào workspace thì không được mời dưới dạng owner
