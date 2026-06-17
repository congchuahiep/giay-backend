Bạn hãy xem một app của một dự án django này: D:\Project\thoikhoabieu\workspace

Nó là một app phục vụ cho chức năng tenant-based! Hiện tại tôi đang làm dự án bằng axum. Dựa vào cách triển khai trên, tôi có thể tạo một hệ thống tenant based mạnh mẽ như vậy không??? Cho phép tôi có thể có một workspace có nhiều người dùng làm việc ở trong, các role của mỗi thành viên trong workspace, lời mời thành viên vào workspace, các chức năng util để tạo model/view có thể có bound workspace???

Chưa triển khai, hãy phân tích tenant based của dự án django, và gợi ý cách triển khai (vì vốn dự án của tôi tại đây chưa có 1 cái gì cả)

Hiện tại dự án của tôi đang sử dụng SeaORM bản 2.0, bạn hãy tra cứu cách nó tạo migrate: https://www.sea-ql.org/SeaORM/docs/migration/writing-migration/
